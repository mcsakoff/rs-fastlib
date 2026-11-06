use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::base::instruction::Instruction;
use crate::base::types::{Dictionary, Operator, Presence, Template, TypeRef};
use crate::base::value::ValueType;
use crate::{Error, Result};

/// Maximum number of bits in a presence map segment.
pub const MAX_PMAP_BITS: usize = 63;

/// Stores template definitions and global processing context.
pub struct Definitions {
    pub(crate) templates: Vec<Rc<Template>>,
    pub(crate) templates_by_id: HashMap<u32, Rc<Template>>,
    pub(crate) templates_by_name: HashMap<String, Rc<Template>>,
    pub(crate) template_id_instruction: Rc<Instruction>,
}

impl Definitions {
    pub(crate) fn new_from_templates(ts: Vec<Template>) -> Result<Self> {
        let mut templates = Vec::with_capacity(ts.len());
        let mut templates_by_id = HashMap::with_capacity(ts.len());
        let mut templates_by_name = HashMap::with_capacity(ts.len());
        for t in ts {
            let t = Rc::new(t);
            if t.id != 0 {
                templates_by_id.insert(t.id, t.clone());
            }
            if !t.name.is_empty() {
                templates_by_name.insert(t.name.clone(), t.clone());
            }
            templates.push(t);
        }

        let template_id_instruction = Rc::new(Instruction {
            id: 0,
            name: "__template_id__".to_string(),
            value_type: ValueType::UInt32,
            presence: Presence::Mandatory,
            operator: Operator::Copy,
            initial_value: None,
            instructions: Vec::new(),
            dictionary: Dictionary::Global,
            key: Rc::from("__template_id__"),
            type_ref: TypeRef::Any,
            has_pmap: Cell::new(false),
        });

        let definitions = Self {
            templates,
            templates_by_id,
            templates_by_name,
            template_id_instruction,
        };
        definitions.finalize()?;
        Ok(definitions)
    }

    pub fn new_from_xml(text: &str) -> Result<Self> {
        let doc = roxmltree::Document::parse(text)?;
        let root = doc
            .root()
            .first_child()
            .ok_or_else(|| Error::Static("no root element found".to_string()))?;
        if root.tag_name().name() != "templates" {
            return Err(Error::Static("<templates/> node not found".to_string()));
        }
        let mut templates = Vec::new();
        for child in root.children() {
            if child.is_element() {
                templates.push(Template::from_node(child)?);
            }
        }
        Self::new_from_templates(templates)
    }

    // After generating the templates we have to go through all the instructions and set flags
    // for structures that must have a presence map. That can only be done when whole
    // templates structure is generated.
    fn finalize(&self) -> Result<()> {
        for tpl in &self.templates {
            let bits = self.count_presence_map_bits(&tpl.instructions)?;
            // The template segment also contains the template identifier bit
            Self::check_segment_size(&tpl.name, bits + 1)?;
            tpl.pmap_bits.set(Some(bits));
        }
        Ok(())
    }

    // Presence map is stored in `u64`, so a segment can't use more than 63 bits.
    fn check_segment_size(name: &str, bits: usize) -> Result<()> {
        if bits > MAX_PMAP_BITS {
            return Err(Error::Static(format!(
                "'{name}' requires {bits} presence map, at most {MAX_PMAP_BITS} supported"
            )));
        }
        Ok(())
    }

    // Count presence map bits the instruction occupy in the enclosing segment.
    // No early exit! Must iterate over all items because presence_map_bits() also initializes has_pmap bit.
    fn count_presence_map_bits(&self, instructions: &[Instruction]) -> Result<usize> {
        let mut pmap_bits = 0;
        for i in instructions {
            pmap_bits += self.presence_map_bits(i)?;
        }
        Ok(pmap_bits)
    }

    // Initializes `has_pmap` of the instructions and returns the number of presence map bits it occupies
    // in the enclosing segment.
    fn presence_map_bits(&self, instr: &Instruction) -> Result<usize> {
        match instr.value_type {
            ValueType::Group => {
                let inner = self.count_presence_map_bits(&instr.instructions)?;
                Self::check_segment_size(&instr.name, inner)?;
                instr.has_pmap.set(inner > 0);
                // If a ::Group field is optional, it will occupy a single bit in the presence map.
                return Ok(usize::from(instr.is_optional()));
            }
            ValueType::Sequence => {
                let length = instr.instructions.first().ok_or_else(|| {
                    Error::Static(format!("sequence '{}' has no length field", instr.name))
                })?;
                let inner = self.count_presence_map_bits(&instr.instructions[1..])?;
                Self::check_segment_size(&instr.name, inner)?;
                instr.has_pmap.set(inner > 0);
                // For ::Sequence its length field show if the sequence has a bit in the presence map.
                return self.presence_map_bits(length);
            }
            ValueType::TemplateReference => {
                if instr.name.is_empty() {
                    // Dynamic template ref doesn't need a presence map bit.
                    return Ok(0);
                }
                // Static template ref occupies the same bits as the corresponding template.
                let template = self
                    .templates_by_name
                    .get(&instr.name)
                    .ok_or_else(|| Error::Static(format!("template '{}' not found", instr.name)))?;
                return template.pmap_bits.get().ok_or_else(|| {
                    Error::Static(format!(
                        "template '{}' not initialized yet; consider reordering templates",
                        instr.name
                    ))
                });
            }
            ValueType::Decimal => {
                let inner = self.count_presence_map_bits(&instr.instructions)?;
                instr.has_pmap.set(inner > 0);
                if instr.operator == Operator::None {
                    // Subcomponents occupy their own bits
                    return Ok(inner);
                }
            }
            // Otherwise, fall-though and check the field's operator (like for normal field).
            _ => {}
        }
        let bit = match instr.operator {
            // If a field (is mandatory and) has no field operator, it will not occupy any bit in the presence map
            // and its value must always appear in the stream.
            // Delta is always present in the stream, so doesn't need a presence bit.
            Operator::None | Operator::Delta => false,
            // Always require a presence bit.
            Operator::Default | Operator::Copy | Operator::Increment | Operator::Tail => true,
            // An optional field with the constant operator will occupy a single bit.
            Operator::Constant => instr.is_optional(),
        };
        Ok(usize::from(bit))
    }
}
