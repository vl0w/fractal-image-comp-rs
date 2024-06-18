use crate::image::Size;
use crate::model::Transformation;

#[derive(Debug, Clone)]
pub struct Compressed {
    /// The size of the compressed image
    pub size: Size,
    
    /// All [transformations](Transformation) to reconstruct the image
    pub transformations: Vec<Transformation>,
}