// TODO elaborate with documentation from spec
#[derive(Debug)]
pub struct Annotation {
    pub type_index: u16,
    pub value_pairs: Vec<ElementPairs>,
}
// TODO elaborate with documentation from spec
#[derive(Debug)]
pub struct ElementPairs(pub u16, pub ElementValue);

// TODO elaborate with documentation from spec
#[derive(Debug)]
pub enum ElementValue {
    ConstIndex(u16),
    EnumConst(u16, u16),
    ClassIndex(u16),
    AnnotationValue(Annotation),
    Array(Vec<ElementValue>),
}

#[derive(Debug)]
pub struct ParameterAnnotations(Vec<Annotation>);
