use crate::coords;
use crate::image::Coords;

/// Represents a region of an image (with size `image_size`) of size `block_size`
/// at position `coords`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Block {
    pub block_size: u32,
    pub image_size: u32,
    pub origin: Coords,
}

impl Block {
    pub fn indices(&self) -> impl Iterator<Item=(usize, Coords)> {
        let mut indices: Vec<(usize, Coords)> = Vec::with_capacity(self.block_size.pow(2) as usize);
        for i in 0..self.block_size {
            for j in 0..self.block_size {
                let index = (self.origin.y * self.image_size + self.origin.x + self.image_size * i + j) as usize;
                indices.push((index, coords!(self.origin.x + j, self.origin.y + i)))
            }
        }

        indices.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::coords;
    use super::*;

    #[test]
    fn get_indices() {
        //  0   1   2   3   4   5   6   7   8   9
        // 10  11  12  13  14  15  16  17  18  19
        // 20  21  22  23  24  25  26  27  28  29
        // 30  31  32  33  34  35  36  37  38  39
        // 40  41  42  43  44  45  46  47  48  49
        // 50  51  52  53  54  55  56  57  58  59
        // 60  61  62  63  64  65  66  67  68  69
        // 70  71  72  73  74  75  76  77  78  79
        // 80  81  82  83  84  85  86  87  88  89
        // 90  91  92  93  94  95  96  97  98  99

        let block = Block {
            block_size: 3,
            image_size: 10,
            origin: coords!(2, 3),
        };

        assert_eq!(vec![
            (32, coords!(2, 3)), (33, coords!(3, 3)), (34, coords!(4, 3)),
            (42, coords!(2, 4)), (43, coords!(3, 4)), (44, coords!(4, 4)),
            (52, coords!(2, 5)), (53, coords!(3, 5)), (54, coords!(4, 5))],
                   block.indices().collect::<Vec<_>>());
    }
}