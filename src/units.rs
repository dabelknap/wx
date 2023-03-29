pub mod temperature {
    pub fn f2c(temp_f: f32) -> f32 {
        (temp_f - 32.0) * 5.0 / 9.0
    }

    pub fn c2f(temp_c: f32) -> f32 {
        temp_c * 9.0 / 5.0 + 32.0
    }

    #[test]
    fn test_temperature() {
        assert_eq!(f2c(212.0), 100.0);
        assert_eq!(f2c(32.0), 0.0);
        assert_eq!(c2f(0.0), 32.0);
        assert_eq!(c2f(100.0), 212.0);
    }
}

pub mod speed {
    const KPM: f32 = 0.621371;

    pub fn kph2mph(kph: f32) -> f32 {
        kph / KPM
    }

    pub fn mph2kph(mph: f32) -> f32 {
        mph * KPM
    }
}

pub mod direction {
    const COMPASS: [&str; 16] = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    pub fn degree_to_compass<'a>(deg: f32) -> &'a str {
        let deg = (deg % 360.0) + 360.0;
        let val = (deg / 22.5 + 0.5) as usize;
        let idx = val % 16;
        COMPASS[idx]
    }

    #[test]
    fn test_degree_to_compass() {
        assert_eq!(degree_to_compass(0.0), "N");
        assert_eq!(degree_to_compass(90.0), "E");
        assert_eq!(degree_to_compass(180.0), "S");
        assert_eq!(degree_to_compass(270.0), "W");
        assert_eq!(degree_to_compass(360.0), "N");
    }
}
