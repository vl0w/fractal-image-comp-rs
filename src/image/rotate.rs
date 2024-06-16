use crate::image::{Image, Pixel, Size};

pub trait IntoRotated<'a, I> {
    fn rot(&'a self, rotation: Rotation) -> Rotated<'a, I>;

    fn rot_0(&'a self) -> Rotated<'a, I> {
        self.rot(Rotation::By0)
    }

    fn rot_90(&'a self) -> Rotated<'a, I> {
        self.rot(Rotation::By90)
    }

    fn rot_180(&'a self) -> Rotated<'a, I> {
        self.rot(Rotation::By180)
    }

    fn rot_270(&'a self) -> Rotated<'a, I> {
        self.rot(Rotation::By270)
    }
}

impl<'a, I> IntoRotated<'a, I> for I
where
    I: Image + 'a,
{
    fn rot(&'a self, rotation: Rotation) -> Rotated<'a, I> {
        Rotated {
            image: self,
            rotation,
        }
    }
}

pub enum Rotation {
    By0,
    By90,
    By180,
    By270,
}

pub struct Rotated<'a, I> {
    image: &'a I,
    rotation: Rotation,
}

impl<'a, I> Image for Rotated<'a, I>
where
    I: Image + 'a,
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

#[cfg(test)]
mod tests {
    use crate::image::rotate::IntoRotated;
    use crate::image::{Image, Size};
    use crate::testutils::FakeImage;

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
