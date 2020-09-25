pub trait Arith {
    fn wrapping_add_assign(&mut self, other: &Self);
    fn wrapping_sub_assign(&mut self, other: &Self);
    fn wrapping_mul_assign(&mut self, other: &Self);
    fn wrapping_div_assign(&mut self, other: &Self);

    fn wrapping_add(self, other: &Self)->Self;
    fn wrapping_sub(self, other: &Self)->Self;
    fn wrapping_mul(self, other: &Self)->Self;
    fn wrapping_div(self, other: &Self)->Self;
    fn wrapping_neg(self)->Self;
}

pub trait ArithEx {
    fn wrapping_rem_assign(&mut self, other: &Self);
    fn wrapping_shl_assign(&mut self, other: u32);
    fn wrapping_shr_assign(&mut self, other: u32);

    fn wrapping_rem(&mut self, other: &Self) -> Self;
    fn wrapping_shl(&mut self, other: u32) -> Self;
    fn wrapping_shr(&mut self, other: u32) -> Self;    
}

pub trait BoolLogic {
    fn wrapping_and_assign(&mut self, other: &Self);
    fn wrapping_or_assign(&mut self, other: &Self);
    fn wrapping_xor_assign(&mut self, other: &Self);

    fn wrapping_and(&mut self, other: &Self) -> Self;
    fn wrapping_or(&mut self, other: &Self) -> Self;
    fn wrapping_xor(&mut self, other: &Self) -> Self;
}


pub trait FromPrimitive {
    fn from_bool(n:bool) -> Self;
    fn from_u8(n:bool) -> Self;
    fn from_u16(n:bool) -> Self;
    fn from_u32(n:bool) -> Self;
    fn from_u64(n:bool) -> Self;
    fn from_u128(n:bool) -> Self;
    fn from_i8(n:bool) -> Self;
    fn from_i16(n:bool) -> Self;
    fn from_i32(n:bool) -> Self;
    fn from_i64(n:bool) -> Self;
    fn from_i128(n:bool) -> Self;
}

pub trait TryToPrimitive {
    fn try_to_bool(&self) -> Option<bool>;
    fn try_to_u8(&self) -> Option<u8>;
    fn try_to_u16(&self) -> Option<u16>;
    fn try_to_u32(&self) -> Option<u32>;
    fn try_to_u64(&self) -> Option<u64>;
    fn try_to_u128(&self) -> Option<u128>;
    fn try_to_i8(&self) -> Option<i8>;
    fn try_to_i16(&self) -> Option<i16>;
    fn try_to_i32(&self) -> Option<i32>;
    fn try_to_i64(&self) -> Option<i64>;
    fn try_to_i128(&self) -> Option<i128>;
}

