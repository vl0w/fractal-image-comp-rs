use crate::model::Transformation;

#[derive(Debug, Clone)]
pub struct Compressed {
    /// The width of the compressed image
    pub width: u32,
    
    /// The height of the compressed image
    pub height: u32,
    
    /// All [transformations](Transformation) to reconstruct the image
    pub transformations: Vec<Transformation>,
}