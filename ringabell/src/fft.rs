// 복소수 구조체 정의
#[derive(Clone, Copy, Debug)]
pub struct Complex {
    re: f32, // 실수부 (Real part)
    im: f32, // 허수부 (Imaginary part)
}

// 복소수 연산 구현
impl Complex {
    pub fn new(re: f32, im: f32) -> Complex {
        Complex { re, im }
    }
    
    // 복소수 덧셈
    fn add(self, other: Complex) -> Complex {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    // 복소수 뺄셈
    fn sub(self, other: Complex) -> Complex {
        Complex {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    // 복소수 곱셈
    fn mul(self, other: Complex) -> Complex {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    // 복소수의 켤레 복소수
    fn conjugate(self) -> Complex {
        Complex {
            re: self.re,
            im: -self.im,
        }
    }

    // 극좌표 형태에서 복소수 생성 (r * e^(i*theta))
    fn from_polar(r: f32, theta: f32) -> Complex {
        Complex {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    // norm (크기) 계산
    pub fn norm(self) -> f32 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

// 재귀적인 FFT 함수 구현
pub fn fft(input: &mut [Complex]) {
    let n = input.len();

    // 입력 크기가 1이면 재귀 종료
    if n <= 1 {
        return;
    }

    // 입력을 짝수와 홀수 인덱스로 분할
    let mut even = input.iter().step_by(2).cloned().collect::<Vec<_>>();
    let mut odd = input.iter().skip(1).step_by(2).cloned().collect::<Vec<_>>();

    // 재귀적으로 FFT 수행
    fft(&mut even);
    fft(&mut odd);

    // 합치는 단계
    for k in 0..n / 2 {
        // 회전 인자 (Twiddle Factor) 계산
        let t = Complex::from_polar(1.0, -2.0 * std::f32::consts::PI * k as f32 / n as f32).mul(odd[k]);
        input[k] = even[k].add(t); // 상부 절반
        input[k + n / 2] = even[k].sub(t); // 하부 절반
    }
}
