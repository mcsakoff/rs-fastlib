use std::rc::Rc;

use rustc_hash::FxHashMap as HashMap;

use crate::base::instruction::Instruction;
use crate::base::types::{Dictionary, Operator, Template, TypeRef};
use crate::base::value::ValueType;
use crate::common::context::DictionaryType;
use crate::utils::xml::td_children;
use crate::{Error, Result};

/// Number of presence map bits that fit into the `u64` bitmap; longer presence maps take a slower path.
pub const MAX_PMAP_BITS: usize = 63;

/// Stores template definitions and global processing context.
pub struct Definitions {
    pub(crate) templates: Vec<Template>,
    pub(crate) templates_by_id: HashMap<u32, usize>,
    // Templates by local name; `None` if the name is ambiguous (used in different namespaces).
    templates_by_name: HashMap<String, Option<usize>>,
    // Templates by (namespace URI, local name).
    qualified_templates: HashMap<(Rc<str>, String), usize>,
    pub(crate) template_id_instruction: Instruction,
    // Message segments (and dynamic template references) may need presence maps longer than `MAX_PMAP_BITS`.
    pub(crate) long_template_pmap: bool,
    // Indexes of application types by (namespace URI, local name); index 0 is reserved for special type `any`.
    type_indices: HashMap<(Rc<str>, Rc<str>), u32>,
}

impl Definitions {
    pub(crate) fn new_from_templates(mut templates: Vec<Template>) -> Result<Self> {
        let mut templates_by_id = HashMap::default();
        let mut templates_by_name = HashMap::default();
        let mut qualified_templates = HashMap::default();
        let mut type_indices = HashMap::default();
        for (i, t) in templates.iter_mut().enumerate() {
            t.index =
                u32::try_from(i).map_err(|_| Error::Static("too many templates".to_string()))?;
            if let Some(id) = t.id {
                templates_by_id.insert(id, i);
            }
            if qualified_templates
                .insert((t.ns.clone(), t.name.clone()), i)
                .is_some()
            {
                return Err(Error::Static(format!(
                    "duplicate template name '{}' (namespace '{}')",
                    t.name, t.ns
                )));
            }
            templates_by_name
                .entry(t.name.clone())
                .and_modify(|e| *e = None)
                .or_insert(Some(i));
            Self::index_type_ref(&t.type_ref, &mut type_indices);
            Self::index_type_refs(&t.instructions, &mut type_indices);
        }

        let template_id_instruction = Instruction::new_template_id();

        let mut definitions = Self {
            templates,
            templates_by_id,
            templates_by_name,
            qualified_templates,
            template_id_instruction,
            long_template_pmap: false,
            type_indices,
        };
        definitions.long_template_pmap = definitions.finalize()?;
        Ok(definitions)
    }

    fn index_type_ref(type_ref: &TypeRef, type_indices: &mut HashMap<(Rc<str>, Rc<str>), u32>) {
        if let TypeRef::ApplicationType(ns, name) = type_ref {
            let next = u32::try_from(type_indices.len() + 1).unwrap_or(u32::MAX);
            type_indices
                .entry((ns.clone(), name.clone()))
                .or_insert(next);
        }
    }

    fn index_type_refs(
        instructions: &[Instruction],
        type_indices: &mut HashMap<(Rc<str>, Rc<str>), u32>,
    ) {
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
            TypeRef::ApplicationType(ns, name) => {
                self.type_indices.get(&(ns.clone(), name.clone())).copied()
            }
        }
    }

    #[inline]
    pub(crate) fn template_by_id(&self, id: u32) -> Result<&Template> {
        match self.templates_by_id.get(&id) {
            Some(&i) => Ok(&self.templates[i]),
            None => Err(Error::Dynamic(format!("Unknown template id: {id}"))), // [ErrD09]
        }
    }

    // Returns the template by its local name.
    #[inline]
    pub(crate) fn template_by_name(&self, name: &str) -> Result<&Template> {
        match self.templates_by_name.get(name) {
            Some(Some(i)) => Ok(&self.templates[*i]),
            Some(None) => Err(Error::Dynamic(format!(
                "Ambiguous template name: {name}; it is used in several namespaces"
            ))),
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
        let root = doc.root_element();
        // The root element is either <templates> or a single <template>.
        let templates = match root.tag_name().name() {
            "templates" => {
                // The dictionary for templates that do not specify their own.
                let dictionary = root
                    .attribute("dictionary")
                    .map_or(Dictionary::Global, Dictionary::from_str);
                td_children(&root)
                    .map(|child| Template::from_node(child, &dictionary))
                    .collect::<Result<Vec<_>>>()?
            }
            "template" => vec![Template::from_node(root, &Dictionary::Global)?],
            _ => {
                return Err(Error::Static(
                    "<templates/> or <template/> root element not found".to_string(),
                ));
            }
        };
        Self::new_from_templates(templates)
    }

    // After generating the templates we have to go through all the instructions and set flags
    // for structures that must have a presence map. That can only be done when whole
    // templates structure is generated.
    // Returns `true` if any template segment needs a presence map longer than `MAX_PMAP_BITS`.
    fn finalize(&self) -> Result<bool> {
        // Templates being processed, to detect cyclic static template references.
        let mut visiting = vec![false; self.templates.len()];
        let mut long_pmap = false;
        for index in 0..self.templates.len() {
            // The template segment also contains the template identifier bit.
            long_pmap |= self.template_pmap_bits(index, &mut visiting)? + 1 > MAX_PMAP_BITS;
        }

        let mut slots = SlotResolver::default();
        // All template identifiers use the copy operator in the `global` dictionary with a common key.
        slots.resolve(&self.template_id_instruction, &Dictionary::Global);
        for tpl in &self.templates {
            slots.resolve_all(&tpl.instructions, &tpl.dictionary);
        }
        Ok(long_pmap)
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
        tpl.pmap_bits.set(Some(bits));
        Ok(bits)
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
                instr.has_pmap.set(inner > 0);
                instr.long_pmap.set(inner > MAX_PMAP_BITS);
                // If a ::Group field is optional, it will occupy a single bit in the presence map.
                return Ok(usize::from(instr.is_optional()));
            }
            ValueType::Sequence => {
                let length = instr.instructions.first().ok_or_else(|| {
                    Error::Static(format!("sequence '{}' has no length field", instr.name))
                })?;
                let inner = self.count_presence_map_bits(&instr.instructions[1..], visiting)?;
                instr.has_pmap.set(inner > 0);
                instr.long_pmap.set(inner > MAX_PMAP_BITS);
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
                    .qualified_templates
                    .get(&(instr.ns.clone(), instr.name.clone()))
                    .ok_or_else(|| {
                        Error::Static(format!(
                            "template '{}' (namespace '{}') not found",
                            instr.name, instr.ns
                        ))
                    })?; // [ERR D8]
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
type StaticKey = (Option<Rc<str>>, Rc<str>, Rc<str>);

#[derive(Default)]
struct SlotResolver {
    // (dictionary name or `None` for global, key namespace URI, key) -> static slot
    statics: HashMap<StaticKey, u32>,
    // (key namespace URI, key) -> key index in `templates` and `type` dictionaries
    keys: HashMap<(Rc<str>, Rc<str>), u32>,
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
                instr.dictionary_slot.set(self.slot(dictionary, instr));
                // Subcomponents inherits the dictionary of the decimal field unless their operators set their own.
                self.resolve_all(&instr.instructions, dictionary);
            }
            _ => {
                instr.dictionary_slot.set(self.slot(dictionary, instr));
            }
        }
    }

    // Keys are names, so two keys are equal iff their namespace URIs and local names are equal.
    fn slot(&mut self, dictionary: &Dictionary, instr: &Instruction) -> DictionaryType {
        let name = match dictionary {
            Dictionary::Template => return DictionaryType::Template(self.key(instr)),
            Dictionary::Type => return DictionaryType::Type(self.key(instr)),
            Dictionary::Inherit | Dictionary::Global => None,
            Dictionary::UserDefined(name) => Some(name.clone()),
        };
        let next = u32::try_from(self.statics.len()).unwrap_or(u32::MAX);
        let entry = (name, instr.ns.clone(), instr.key.clone());
        DictionaryType::Static(*self.statics.entry(entry).or_insert(next))
    }

    fn key(&mut self, instr: &Instruction) -> u32 {
        let next = u32::try_from(self.keys.len()).unwrap_or(u32::MAX);
        *self
            .keys
            .entry((instr.ns.clone(), instr.key.clone()))
            .or_insert(next)
    }
}
