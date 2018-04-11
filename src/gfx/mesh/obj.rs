
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{BufReader, Error as IoError, Read};

use hal::Backend;
use mesh::{Mesh, MeshBuilder, Position, Normal, TexCoord};
use render::{Factory, Error as RenderError};
use obj::{Obj, SimplePolygon};

use asset::Asset;

#[derive(Debug)]
pub enum ObjError {
    Io(IoError),
    Render(RenderError),
}

impl Display for ObjError {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        match *self {
            ObjError::Io(ref err) => {
                write!(fmt, "Io error: {}", err)
            }
            ObjError::Render(ref err) => {
                write!(fmt, "Render error: {}", err)
            }
        }
    }
}

impl Error for ObjError {
    fn description(&self) -> &str {
        match *self {
            ObjError::Io(ref err) => err.description(),
            ObjError::Render(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ObjError::Io(ref err) => Some(err),
            ObjError::Render(ref err) => Some(err),
        }
    }
}

pub struct ObjFormat;

impl<B> Asset<ObjFormat, Factory<B>> for Mesh<B>
where
    B: Backend,
{
    type Error = ObjError;

    fn load<R>(read: R, _format: ObjFormat, factory: &mut Factory<B>) -> Result<Self, ObjError>
    where
        R: Read,
    {
        let obj: Obj<SimplePolygon> = Obj::load_buf(&mut BufReader::new(read)).map_err(ObjError::Io)?;
        let mut indices = Vec::new();
        let positions = obj.position.iter().cloned().map(Position).collect::<Vec<_>>();
        let mut texcoords = None;
        let mut normals = None;

        {
            let mut texture = |index, value| {
                let texcoords = texcoords.get_or_insert_with(|| Vec::new());
                let len = texcoords.len();
                texcoords.extend((len .. index + 1).map(|_| value));
                texcoords[index] = value;
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
        if let Some(texcoords) = texcoords {
            builder.add_vertices(texcoords);
        };

        builder.build(factory).map_err(ObjError::Render)
    }
}

