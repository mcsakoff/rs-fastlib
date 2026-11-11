use std::rc::Rc;

use rustc_hash::FxHashMap as HashMap;

use crate::base::instruction::Instruction;
use crate::base::types::{Dictionary, Operator, Template, TypeRef};
use crate::base::value::ValueType;
use crate::common::context::DictionaryType;
use crate::{Error, Result};

/// Maximum number of bits in a presence map segment.
pub const MAX_PMAP_BITS: usize = 63;

/// Stores template definitions and global processing context.
pub struct Definitions {
    pub(crate) templates: Vec<Template>,
    pub(crate) templates_by_id: HashMap<u32, usize>,
    pub(crate) templates_by_name: HashMap<String, usize>,
    pub(crate) template_id_instruction: Instruction,
    // Indexes of application types; index 0 is reserved for special type `any`.
    type_indices: HashMap<Rc<str>, u32>,
}

impl Definitions {
    pub(crate) fn new_from_templates(mut templates: Vec<Template>) -> Result<Self> {
        let mut templates_by_id = HashMap::default();
        let mut templates_by_name = HashMap::default();
        let mut type_indices = HashMap::default();
        for (i, t) in templates.iter_mut().enumerate() {
            t.index =
                u32::try_from(i).map_err(|_| Error::Static("too many templates".to_string()))?;
            if t.id != 0 {
                templates_by_id.insert(t.id, i);
            }
            if !t.name.is_empty() {
                templates_by_name.insert(t.name.clone(), i);
            }
            Self::index_type_ref(&t.type_ref, &mut type_indices);
            Self::index_type_refs(&t.instructions, &mut type_indices);
        }

        let template_id_instruction = Instruction::new_template_id();

        let definitions = Self {
            templates,
            templates_by_id,
            templates_by_name,
            template_id_instruction,
            type_indices,
        };
        definitions.finalize()?;
        Ok(definitions)
    }

    fn index_type_ref(type_ref: &TypeRef, type_indices: &mut HashMap<Rc<str>, u32>) {
        if let TypeRef::ApplicationType(name) = type_ref {
            let next = u32::try_from(type_indices.len() + 1).unwrap_or(u32::MAX);
            type_indices.entry(name.clone()).or_insert(next);
        }
    }

    fn index_type_refs(instructions: &[Instruction], type_indices: &mut HashMap<Rc<str>, u32>) {
        for i in instructions {
            Self::index_type_ref(&i.type_ref, type_indices);
            Self::index_type_refs(&i.instructions, type_indices);
        }
    }

    /// Returns index of application type, or `None` for the special type `any`.
    #[inline]
    pub(crate) fn type_index(&self, type_ref: &TypeRef) -> Option<u32> {
        match type_ref {
            TypeRef::Any => None,
            TypeRef::ApplicationType(name) => self.type_indices.get(name).copied(),
        }
    }

    #[inline]
    pub(crate) fn template_by_id(&self, id: u32) -> Result<&Template> {
        match self.templates_by_id.get(&id) {
            Some(&i) => Ok(&self.templates[i]),
            None => Err(Error::Dynamic(format!("Unknown template id: {id}"))), // [ErrD09]
        }
    }

    #[inline]
    pub(crate) fn template_by_name(&self, name: &str) -> Result<&Template> {
        match self.templates_by_name.get(name) {
            Some(&i) => Ok(&self.templates[i]),
            None => Err(Error::Dynamic(format!("Unknown template: {name}"))), // [ErrD09]
        }
    }

    // Returns the template of a static template reference instruction (resolved at load time).
    #[inline]
    pub(crate) fn static_template_ref(&self, instr: &Instruction) -> &Template {
        &self.templates[instr.template_index.get()]
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
        // The dictionary for templates that do not specify their own.
        let dictionary = root
            .attribute("dictionary")
            .map_or(Dictionary::Global, Dictionary::from_str);
        let mut templates = Vec::new();
        for child in root.children() {
            if child.is_element() {
                templates.push(Template::from_node(child, &dictionary)?);
            }
        }
        Self::new_from_templates(templates)
    }

    // After generating the templates we have to go through all the instructions and set flags
    // for structures that must have a presence map. That can only be done when whole
    // templates structure is generated.
    fn finalize(&self) -> Result<()> {
        // Templates being processed, to detect cyclic static template references.
        let mut visiting = vec![false; self.templates.len()];
        for index in 0..self.templates.len() {
            self.template_pmap_bits(index, &mut visiting)?;
        }

        let mut slots = SlotResolver::default();
        // All template identifiers use the copy operator in the `global` dictionary with a common key.
        slots.resolve(&self.template_id_instruction, &Dictionary::Global);
        for tpl in &self.templates {
            slots.resolve_all(&tpl.instructions, &tpl.dictionary);
        }
        Ok(())
    }

    // Returns the number of presence map bits the template instructions occupy (without the template
    // identifier bit). Computed on first call, so static references can refer to templates defined later.
    fn template_pmap_bits(&self, index: usize, visiting: &mut [bool]) -> Result<usize> {
        let tpl = &self.templates[index];
        if let Some(bits) = tpl.pmap_bits.get() {
            return Ok(bits);
        }
        if visiting[index] {
            return Err(Error::Static(format!(
                "template '{}' statically references itself",
                tpl.name
            )));
        }
        visiting[index] = true;
        let bits = self.count_presence_map_bits(&tpl.instructions, visiting)?;
        visiting[index] = false;
        // The template segment also contains the template identifier bit
        Self::check_segment_size(&tpl.name, bits + 1)?;
        tpl.pmap_bits.set(Some(bits));
        Ok(bits)
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
    fn count_presence_map_bits(
        &self,
        instructions: &[Instruction],
        visiting: &mut [bool],
    ) -> Result<usize> {
        let mut pmap_bits = 0;
        for i in instructions {
            pmap_bits += self.presence_map_bits(i, visiting)?;
        }
        Ok(pmap_bits)
    }

    // Initializes `has_pmap` of the instructions and returns the number of presence map bits it occupies
    // in the enclosing segment.
    fn presence_map_bits(&self, instr: &Instruction, visiting: &mut [bool]) -> Result<usize> {
        match instr.value_type {
            ValueType::Group => {
                let inner = self.count_presence_map_bits(&instr.instructions, visiting)?;
                Self::check_segment_size(&instr.name, inner)?;
                instr.has_pmap.set(inner > 0);
                // If a ::Group field is optional, it will occupy a single bit in the presence map.
                return Ok(usize::from(instr.is_optional()));
            }
            ValueType::Sequence => {
                let length = instr.instructions.first().ok_or_else(|| {
                    Error::Static(format!("sequence '{}' has no length field", instr.name))
                })?;
                let inner = self.count_presence_map_bits(&instr.instructions[1..], visiting)?;
                Self::check_segment_size(&instr.name, inner)?;
                instr.has_pmap.set(inner > 0);
                // For ::Sequence its length field show if the sequence has a bit in the presence map.
                return self.presence_map_bits(length, visiting);
            }
            ValueType::TemplateReference => {
                if instr.name.is_empty() {
                    // Dynamic template ref doesn't need a presence map bit.
                    return Ok(0);
                }
                // Static template ref occupies the same bits as the corresponding template.
                let index = *self
                    .templates_by_name
                    .get(&instr.name)
                    .ok_or_else(|| Error::Static(format!("template '{}' not found", instr.name)))?;
                instr.template_index.set(index);
                return self.template_pmap_bits(index, visiting);
            }
            ValueType::Decimal => {
                let inner = self.count_presence_map_bits(&instr.instructions, visiting)?;
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

// Assigns dictionary slots to instructions.
//
// Every template sets its own dictionary (`global` if not specified), so the dictionary of an instruction
// doesn't depend on the place where the template is used and can be resolved statically.
// Only the instance of `template` and `type` dictionaries (current template / application type) is
// known at runtime.
#[derive(Default)]
struct SlotResolver {
    // (dictionary name or `None` for global, key) -> static slot
    statics: HashMap<(Option<Rc<str>>, Rc<str>), u32>,
    // key -> key index in `templates` and `type` dictionaries
    keys: HashMap<Rc<str>, u32>,
}

impl SlotResolver {
    fn resolve_all(&mut self, instructions: &[Instruction], dictionary: &Dictionary) {
        for i in instructions {
            self.resolve(i, dictionary);
        }
    }

    // `dictionary` is the dictionary inherited from the enclosing element.
    fn resolve(&mut self, instr: &Instruction, dictionary: &Dictionary) {
        let dictionary = if instr.dictionary == Dictionary::Inherit {
            dictionary
        } else {
            &instr.dictionary
        };
        match instr.value_type {
            ValueType::Group | ValueType::Sequence => {
                self.resolve_all(&instr.instructions, dictionary);
            }
            // Instructions of the referenced template are resolved on their own.
            ValueType::TemplateReference => {}
            ValueType::Decimal => {
                instr.dictionary_slot.set(self.slot(dictionary, &instr.key));
                // Subcomponents inherits the dictionary of the decimal field unless their operators set their own.
                self.resolve_all(&instr.instructions, dictionary);
            }
            _ => {
                instr.dictionary_slot.set(self.slot(dictionary, &instr.key));
            }
        }
    }

    fn slot(&mut self, dictionary: &Dictionary, key: &Rc<str>) -> DictionaryType {
        let name = match dictionary {
            Dictionary::Template => return DictionaryType::Template(self.key(key)),
            Dictionary::Type => return DictionaryType::Type(self.key(key)),
            Dictionary::Inherit | Dictionary::Global => None,
            Dictionary::UserDefined(name) => Some(name.clone()),
        };
        let next = u32::try_from(self.statics.len()).unwrap_or(u32::MAX);
        DictionaryType::Static(*self.statics.entry((name, key.clone())).or_insert(next))
    }

    fn key(&mut self, key: &Rc<str>) -> u32 {
        let next = u32::try_from(self.keys.len()).unwrap_or(u32::MAX);
        *self.keys.entry(key.clone()).or_insert(next)
    }
}
