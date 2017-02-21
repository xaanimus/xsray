
use super::misc::*;
use std::fmt::{Debug, Formatter};
use std::fmt;

pub trait Shader {
}

impl Debug for Shader {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Shader")
    }
}

pub struct DiffuseShader {
    color: Color3
}

impl DiffuseShader {
    pub fn new(color: Color3) -> DiffuseShader {
        DiffuseShader {
            color: color
        }
    }
}

impl Shader for DiffuseShader {
}
