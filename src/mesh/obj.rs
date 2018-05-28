
use std::io::{BufReader, Read};

use failure::Error;
use hal::{Backend, queue::QueueFamilyId};
use gfx_mesh::{Mesh, MeshBuilder, Position, Normal, TexCoord};
use render::Factory;
use obj::{Obj, SimplePolygon};

use asset::AssetLoader;

pub struct ObjFormat(QueueFamilyId);

impl<B> AssetLoader<Mesh<B>, ObjFormat> for Factory<B>
where
    B: Backend,
{
    type Error = Error;

    fn load<R>(&mut self, ObjFormat(family): ObjFormat, reader: R) -> Result<Mesh<B>, Error>
    where
        R: Read,
    {
        let obj: Obj<SimplePolygon> = Obj::load_buf(&mut BufReader::new(reader))?;
        let mut indices = Vec::new();
        let positions = obj.position.iter().cloned().map(Position).collect::<Vec<_>>();
        let mut coords = None;
        let mut normals = None;

        {
            let mut texture = |index, value| {
                let coords = coords.get_or_insert_with(|| Vec::new());
                let len = coords.len();
                coords.extend((len .. index + 1).map(|_| value));
                coords[index] = value;
            };

            let mut normal = |index, value| {
                let normals = normals.get_or_insert_with(|| Vec::new());
                let len = normals.len();
                normals.extend((len .. index + 1).map(|_| value));
                normals[index] = value;
            };

            let mut triangle = |i: [usize; 3], t: [Option<usize>; 3], n: [Option<usize>; 3]| {
                indices.push(i[0] as u32);
                indices.push(i[1] as u32);
                indices.push(i[2] as u32);

                match (t[0], t[1], t[2]) {
                    (Some(t0), Some(t1), Some(t2)) => {
                        texture(i[0], TexCoord(obj.texture[t0]));
                        texture(i[1], TexCoord(obj.texture[t1]));
                        texture(i[2], TexCoord(obj.texture[t2]));
                    }
                    _ => {
                        unimplemented!()
                    }
                }

                match (n[0], n[1], n[2]) {
                    (Some(n0), Some(n1), Some(n2)) => {
                        normal(i[0], Normal(obj.normal[n0]));
                        normal(i[1], Normal(obj.normal[n1]));
                        normal(i[2], Normal(obj.normal[n2]));
                    }
                    _ => {
                        unimplemented!()
                    }
                }
            };

            for object in &obj.objects {
                for group in &object.groups {
                    for poly in &group.polys {
                        for c in 2 .. poly.len() {
                            triangle([poly[0].0, poly[c-1].0, poly[c].0], [poly[0].1, poly[c-1].1, poly[c].1], [poly[0].2, poly[c-1].2, poly[c].2]);
                        }
                    }
                }
            }
        }

        let mut builder = MeshBuilder::new()
            .with_indices(indices)
            .with_vertices(positions);
        
        if let Some(normals) = normals {
            builder.add_vertices(normals);
        };
        if let Some(coords) = coords {
            builder.add_vertices(coords);
        };

        builder.build(family, self)
    }
}

