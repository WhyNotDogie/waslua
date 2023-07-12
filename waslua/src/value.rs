pub struct LuaValue {
    ty: LuaValueType
}

enum LuaValueType {
    Number(crate::wit::waslua::luatypes::luatypes::Number)
}

impl From<f64> for LuaValue {
    fn from(v: f64) -> LuaValue {
        let x: u64 = u64::from_ne_bytes(v.to_ne_bytes());
        LuaValue { ty: LuaValueType::Number(
            crate::wit::waslua::luatypes::luatypes::Number {
                value: 6
            }
        ) }
    }
}