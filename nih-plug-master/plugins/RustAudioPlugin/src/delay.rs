

pub struct StereoDelay {
    buf: Vec<(f64, f64)>,
    length: usize,
    max_length: usize,
    head: usize,
    delay_ms: i32,
    sample_rate: f32
}

impl StereoDelay {
    pub fn new( delay_ms: i32, max_delay_ms: i32, sample_rate: f32 ) -> StereoDelay {

        let buf_sz = ( ( max_delay_ms as f32 / 1000f32 ) * sample_rate ) as usize;
        let delay_samples = ( ( delay_ms as f32 / 1000f32 ) * sample_rate ) as usize;

        Self {
            buf: vec![ (0f64, 0f64); buf_sz ],
            max_length: buf_sz,
            length: delay_samples,
            head: 0,
            delay_ms,
            sample_rate
        }
    }

    pub fn head( &self ) -> (f64, f64) {
        self.buf[ self.head ]
    }

    pub fn consume( &mut self, sample: ( f64, f64 ) ) {
        self.buf[ self.head ] = sample;
        self.head = (self.head + 1) % self.length;
    }

    pub fn length( &self ) -> usize {
        self.length
    }

    pub fn set_delay( &mut self, delay_ms: i32 ) -> Result<&'static str, &'static str> {

        if delay_ms == self.delay_ms {
            return Ok( "Delay size already correct" )
        }

        let delay_samples = ( ( delay_ms as f32 / 1000f32 ) * self.sample_rate ) as usize;

        if self.max_length >= delay_samples {
            self.delay_ms = delay_ms;
            self.length = delay_samples;
            self.head = 0;

            for i in 0..delay_samples {
                self.buf[ i ] = ( 0f64, 0f64 )
            }

            Ok("Resized delay line")
        }
        else {
            Err("Buffer too small for requested delay")
        }
    }
}