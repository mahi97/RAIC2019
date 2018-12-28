// We will need to work with 2d vertors
#[derive(Copy, Clone, Debug, Default)]
struct Vec2 {
    x: f64,
    y: f64,
}

const VEC2INVALID : Vec2 = Vec2{x:5000.0,y:5000.0};

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn from_polar(mag: f64, theta: AngDeg) -> Self{
        Self {
            x: mag * theta.deg().cos(),
            y: mag * theta.deg().sin(),
        }
    }
    // Finding length of the vector
    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    // Normalizing vector (setting its length to 1)
    fn normalize(self) -> Self {
        self * (1.0 / self.len())
    }

    fn dist(&self, other: Self) -> f64 {
        (*self - other).len()
    }

    fn th(self) -> AngDeg {
        AngDeg{degree:(self.normalize().y).atan2(self.normalize().x * 180.0 / 3.1415)}
    }
}

// Subtraction operation for vectors
impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
}

// Addition for vectors
impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
}

// Multiplying vector by a number
impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, k: f64) -> Self {
        Self::new(self.x * k, self.y * k)
    }
}
