use crate::image::iter::PixelIterator;
use crate::image::{Coords, Image, IterablePixels, Pixel, Size};
use crate::model::Rotation;
use std::sync::Arc;

pub trait IntoRotated<I>
where
    Self: Sized,
{
    fn rot(self, rotation: Rotation) -> Rotated<I>;

    fn rot_0(self) -> Rotated<I> {
        self.rot(Rotation::By0)
    }

    fn rot_90(self) -> Rotated<I> {
        self.rot(Rotation::By90)
    }

    fn rot_180(self) -> Rotated<I> {
        self.rot(Rotation::By180)
    }

    fn rot_270(self) -> Rotated<I> {
        self.rot(Rotation::By270)
    }

    fn all_rotations(self) -> Vec<Rotated<I>>
    where
        Self: Clone,
    {
        vec![
            self.clone().rot_0(),
            self.clone().rot_90(),
            self.clone().rot_180(),
            self.clone().rot_270(),
        ]
    }
}

impl<I> IntoRotated<I> for I
where
    I: Image,
{
    fn rot(self, rotation: Rotation) -> Rotated<I> {
        Rotated {
            image: Arc::new(self),
            rotation,
        }
    }
}

impl<I> IntoRotated<I> for Arc<I>
where
    I: Image,
{
    fn rot(self, rotation: Rotation) -> Rotated<I> {
        Rotated {
            image: self.clone(),
            rotation,
        }
    }
}

#[derive(Clone)]
pub struct Rotated<I> {
    image: Arc<I>,
    pub rotation: Rotation,
}

impl<I> Rotated<I> {
    pub fn inner(&self) -> Arc<I> {
        self.image.clone()
    }
}

impl<I> Image for Rotated<I>
where
    I: Image,
{
    fn get_size(&self) -> Size {
        self.image.get_size()
    }

    fn pixel(&self, x: u32, y: u32) -> Pixel {
        match self.rotation {
            Rotation::By0 => self.image.pixel(x, y),
            Rotation::By90 => self.image.pixel(y, self.get_width() - 1 - x),
            Rotation::By180 => self
                .image
                .pixel(self.get_width() - 1 - x, self.get_height() - 1 - y),
            Rotation::By270 => self.image.pixel(self.get_height() - 1 - y, x),
        }
    }
}

impl<I> IterablePixels for Rotated<I>
where
    I: Image,
{
    fn pixels_enumerated(&self) -> impl Iterator<Item = (Pixel, Coords)> {
        PixelIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::image::rotate::IntoRotated;
    use crate::image::{Image, Size};
    use crate::image::fake::FakeImage;

    #[test]
    fn rotate_squared_by_0() {
        // 0 1
        // 2 3

        let image = FakeImage::squared(2);
        let image = image.rot_0();
        assert_eq!(image.get_size(), Size::new(2, 2));
        assert_eq!(image.pixel(0, 0), 0);
        assert_eq!(image.pixel(1, 0), 1);
        assert_eq!(image.pixel(0, 1), 2);
        assert_eq!(image.pixel(1, 1), 3);
    }

    #[test]
    fn rotate_squared_by_90() {
        // 0 1
        // 2 3

        // 2 0
        // 3 1

        let image = FakeImage::squared(2);
        let image = image.rot_90();
        assert_eq!(image.get_size(), Size::new(2, 2));
        assert_eq!(image.pixel(0, 0), 2);
        assert_eq!(image.pixel(1, 0), 0);
        assert_eq!(image.pixel(0, 1), 3);
        assert_eq!(image.pixel(1, 1), 1);
    }

    #[test]
    fn rotate_squared_by_180() {
        // 0 1
        // 2 3

        // 3 2
        // 1 0

        let image = FakeImage::squared(2);
        let image = image.rot_180();
        assert_eq!(image.get_size(), Size::new(2, 2));
        assert_eq!(image.pixel(0, 0), 3);
        assert_eq!(image.pixel(1, 0), 2);
        assert_eq!(image.pixel(0, 1), 1);
        assert_eq!(image.pixel(1, 1), 0);
    }

    #[test]
    fn rotate_squared_by_270() {
        // 0 1
        // 2 3

        // 1 3
        // 0 2

        let image = FakeImage::squared(2);
        let image = image.rot_270();
        assert_eq!(image.get_size(), Size::new(2, 2));
        assert_eq!(image.pixel(0, 0), 1);
        assert_eq!(image.pixel(1, 0), 3);
        assert_eq!(image.pixel(0, 1), 0);
        assert_eq!(image.pixel(1, 1), 2);
    }
}
