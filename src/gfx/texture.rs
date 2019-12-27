use luminance::pixel::NormRGB8UI;
use luminance::texture::{Dim2, Flat, GenMipmaps, Sampler, Texture};
use luminance_glutin::GlutinSurface;

fn load_from_disk(
  surface: &mut GlutinSurface,
  img: image::RgbImage,
) -> Texture<Flat, Dim2, NormRGB8UI> {
  let (width, height) = img.dimensions();
  let texels = img.into_raw();

  let texture =
    Texture::new(surface, [width, height], 0, Sampler::default()).expect("Texture create error");

  texture.upload_raw(GenMipmaps::No, &texels).unwrap();

  texture
}
