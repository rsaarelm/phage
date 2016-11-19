use num::traits::{ToPrimitive, NumCast};
use std::marker::{PhantomData};
use vec_map::{VecMap};
use std::fmt::{Debug};
use image::{GenericImage, SubImage, Pixel};
use ::geom::{V2, IterTiles};
use super::canvas::{CanvasBuilder, Image};

/// A cache that loads sprites and stores them efficiently indexed by
/// enum constants given by the user.
///
/// Use an enum that contains names for all the sprites as the key type.
pub struct SpriteCache<T> {
    cache: VecMap<Image>,
    // Lock the cache into the specific enum type even though it doesn't
    // show up in the actual implementation, just to keep usage clearer.
    phantom: PhantomData<T>,
}

impl<T: Debug+Copy+SpriteKey> SpriteCache<T> {
    pub fn new() -> SpriteCache<T> {
        SpriteCache {
            cache: VecMap::new(),
            phantom: PhantomData,
        }
    }
    
    /// Try to retrieve an image with the given identifier.
    pub fn get(&self, key: T) -> Option<Image> {
        self.cache.get(SpriteCache::index(key)).map(|&x| x)
    }

    /// Try to retrieve an image some steps ahead of the given key.
    ///
    /// Useful for animations and variants where there are deterministic
    /// consecutive sequences of frames and you don't want to hardcode
    /// every frame as a separate constant.
    pub fn get_nth(&self, key: T, offset: usize) -> Option<Image> {
        self.cache.get(SpriteCache::index(key) + offset).map(|&x| x)
    }

    /// Add an image to the CanvasBuilder atlas for the given identifier key.
    ///
    /// Trying to add multiple sprites with the same key is considered a
    /// bug and will cause a panic.
    pub fn add<P, I>(&mut self, builder: &mut CanvasBuilder, key: T, offset: V2<i32>, image: &I)
        where P: Pixel<Subpixel=u8> + 'static,
              I: GenericImage<Pixel=P>
    {
        let idx = SpriteCache::index(key);
        assert!(self.cache.get(idx).is_none(), format!("Adding sprite {:?} twice", key));

        self.cache.insert(idx, builder.add_image(offset, image));
    }

    /// Add multiple images from a sprite sheet.
    ///
    /// The sheet images are assumed to be in a grid with sprite_size
    /// cells. The sheet is read from top to bottom in left-to-right
    /// rows. Sprites are read and given cache keys corresponding to the
    /// values in the keys Vec as long as there are items left in keys.
    /// If there are more keys in the keys Vec than there are sprites in
    /// the sprite sheet, the function will panic.
    pub fn batch_add<P, I>(&mut self, builder: &mut CanvasBuilder, offset: V2<i32>, sprite_size: V2<u32>,
                           sprite_sheet: &mut I, keys: Vec<T>)
        where P: Pixel<Subpixel=u8> + 'static,
              I: GenericImage<Pixel=P> + 'static
    {
        for (i, rect) in sprite_sheet.tiles(sprite_size).take(keys.len()).enumerate() {
            let sub = SubImage::new(sprite_sheet, rect.mn().0, rect.mn().1, rect.dim().0, rect.dim().1);
            self.add(builder, keys[i], offset, &sub);
        }
    }

    fn index(key: T) -> usize {
        key.to_usize()
    }
}

// TODO: Find a way to convert the type param to usize that can use a
// derive-statment or something, instead of needing the boilerplate of
// implementing this trait...
pub trait SpriteKey {
    fn to_usize(self) -> usize;
}

impl<T: ToPrimitive> SpriteKey for T {
    fn to_usize(self) -> usize {
        NumCast::from(self).unwrap()
    }
}
