pub mod chunk_pos {
    use noise::Vector2;

    const MARKER: u64 = packed(Vector2 {
        x: 1875066,
        y: 1875066,
    });

    pub const fn packed(vec: Vector2<i32>) -> u64 {
        (vec.x as u64 & 4294967295u64) | ((vec.y as u64 & 4294967295u64) << 32)
    }

    pub const fn unpack_x(packed: u64) -> i32 {
        (packed & 4294967295u64) as i32
    }

    pub const fn unpack_z(packed: u64) -> i32 {
        ((packed >> 32) & 4294967295u64) as i32
    }

    pub const fn start_x(vec: Vector2<i32>) -> i32 {
        vec.x << 4
    }

    pub const fn start_y(vec: Vector2<i32>) -> i32 {
        vec.y << 4
    }
}
