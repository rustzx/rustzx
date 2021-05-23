use crate::{AyMode, AymBackend, SoundChip, StereoSample, AY_REGISTER_COUNT};

const TONE_CHANNELS: usize = 3;
const DECIMATE_FACTOR: usize = 8;
const FIR_SIZE: usize = 192;
const DC_FILTER_SIZE: usize = 1024;

#[derive(Default)]
struct ToneChannel {
    tone_period: u16,
    tone_counter: u16,
    tone: usize,
    tone_off_bit: usize,
    noise_off_bit: usize,
    envelope_enabled: bool,
    volume: usize,
    pan_left: f64,
    pan_right: f64,
}

#[derive(Default)]
struct Interpolator {
    c: [f64; 4],
    y: [f64; 4],
}

struct DcFilter {
    sum: f64,
    delay: [f64; DC_FILTER_SIZE],
}

impl Default for DcFilter {
    fn default() -> Self {
        Self {
            sum: 0.0,
            delay: [0.0; DC_FILTER_SIZE],
        }
    }
}

/// Precise AY/YM sound chip generation backend.
///
/// Original code for this backend is derived from `ayumi` C library by Peter Sovietov
/// Link to original repo: https://github.com/true-grue/ayumi
///
/// Uses f64 for computations.
pub struct AymPrecise {
    channels: [ToneChannel; TONE_CHANNELS],

    noise_period: u16,
    noise_counter: u16,
    noise: usize,

    envelope_counter: u16,
    envelope_period: u16,
    envelope_shape: usize,
    envelope_segment: usize,
    envelope: usize,

    dac_table: &'static [f64; 32],
    step: f64,
    x: f64,
    interpolator_left: Interpolator,
    interpolator_right: Interpolator,

    fir_left: [f64; FIR_SIZE * 2],
    fir_right: [f64; FIR_SIZE * 2],
    fir_index: usize,

    dc_left: DcFilter,
    dc_right: DcFilter,
    dc_index: usize,

    left: f64,
    right: f64,

    registers: [u8; AY_REGISTER_COUNT],
    dc_filter: bool,
}

#[rustfmt::skip]
const AY_DAC_TABLE: [f64; 32] = [
    0.0, 0.0,
    0.00999465934234, 0.00999465934234,
    0.0144502937362, 0.0144502937362,
    0.0210574502174, 0.0210574502174,
    0.0307011520562, 0.0307011520562,
    0.0455481803616, 0.0455481803616,
    0.0644998855573, 0.0644998855573,
    0.107362478065, 0.107362478065,
    0.126588845655, 0.126588845655,
    0.20498970016, 0.20498970016,
    0.292210269322, 0.292210269322,
    0.372838941024, 0.372838941024,
    0.492530708782, 0.492530708782,
    0.635324635691, 0.635324635691,
    0.805584802014, 0.805584802014,
    1.0, 1.0
];

#[rustfmt::skip]
const YM_DAC_TABLE: [f64; 32] = [
    0.0, 0.0,
    0.00465400167849, 0.00772106507973,
    0.0109559777218, 0.0139620050355,
    0.0169985503929, 0.0200198367285,
    0.024368657969, 0.029694056611,
    0.0350652323186, 0.0403906309606,
    0.0485389486534, 0.0583352407111,
    0.0680552376593, 0.0777752346075,
    0.0925154497597, 0.111085679408,
    0.129747463188, 0.148485542077,
    0.17666895552, 0.211551079576,
    0.246387426566, 0.281101701381,
    0.333730067903, 0.400427252613,
    0.467383840696, 0.53443198291,
    0.635172045472, 0.75800717174,
    0.879926756695, 1.0
];

static ENVELOPES: [[fn(&mut AymPrecise); 2]; 16] = [
    [AymPrecise::slide_down, AymPrecise::hold_bottom],
    [AymPrecise::slide_down, AymPrecise::hold_bottom],
    [AymPrecise::slide_down, AymPrecise::hold_bottom],
    [AymPrecise::slide_down, AymPrecise::hold_bottom],
    [AymPrecise::slide_up, AymPrecise::hold_bottom],
    [AymPrecise::slide_up, AymPrecise::hold_bottom],
    [AymPrecise::slide_up, AymPrecise::hold_bottom],
    [AymPrecise::slide_up, AymPrecise::hold_bottom],
    [AymPrecise::slide_down, AymPrecise::slide_down],
    [AymPrecise::slide_down, AymPrecise::hold_bottom],
    [AymPrecise::slide_down, AymPrecise::slide_up],
    [AymPrecise::slide_down, AymPrecise::hold_top],
    [AymPrecise::slide_up, AymPrecise::slide_up],
    [AymPrecise::slide_up, AymPrecise::hold_top],
    [AymPrecise::slide_up, AymPrecise::slide_down],
    [AymPrecise::slide_up, AymPrecise::hold_bottom],
];

static ENVELOPE_RESET_TO_MAX: [[bool; 2]; 16] = [
    [true, false],
    [true, false],
    [true, false],
    [true, false],
    [false, false],
    [false, false],
    [false, false],
    [false, false],
    [true, true],
    [true, false],
    [true, false],
    [true, true],
    [false, false],
    [false, true],
    [false, true],
    [false, false],
];

impl AymPrecise {
    fn new(is_ym: bool, clock_rate: f64, sample_rate: usize) -> Self {
        let mut this = Self {
            channels: Default::default(),
            noise_period: 0,
            noise_counter: 0,
            noise: 0,
            envelope_counter: 0,
            envelope_period: 0,
            envelope_shape: 0,
            envelope_segment: 0,
            envelope: 0,
            dac_table: &AY_DAC_TABLE,
            step: 0.0,
            x: 0.0,
            interpolator_left: Default::default(),
            interpolator_right: Default::default(),
            fir_left: [0.0; FIR_SIZE * 2],
            fir_right: [0.0; FIR_SIZE * 2],
            fir_index: 0,
            dc_left: Default::default(),
            dc_right: Default::default(),
            dc_index: 0,
            left: 0.0,
            right: 0.0,
            registers: [0; AY_REGISTER_COUNT],
            dc_filter: false,
        };

        this.step = clock_rate / (sample_rate as f64 * 8f64 * DECIMATE_FACTOR as f64);
        if is_ym {
            this.dac_table = &YM_DAC_TABLE;
        }
        this.noise = 1;
        this.set_envelope(1);
        for i in 0..TONE_CHANNELS {
            this.set_tone(i, 1);
        }
        this
    }

    fn set_pan(&mut self, index: usize, pan: f64, is_eqp: bool) {
        if is_eqp {
            self.channels[index].pan_left = libm::sqrt(1f64 - pan);
            self.channels[index].pan_right = libm::sqrt(pan);
        } else {
            self.channels[index].pan_left = 1f64 - pan;
            self.channels[index].pan_right = pan;
        }
    }

    fn set_tone(&mut self, index: usize, period: u16) {
        let period = period & 0xFFF;
        self.channels[index].tone_period = (period == 0) as u16 | period;
    }

    fn set_noise(&mut self, period: u16) {
        self.noise_period = period & 0x1F;
    }

    fn set_mixer(
        &mut self,
        index: usize,
        tone_enable: bool,
        noise_enable: bool,
        envelope_enabled: bool,
    ) {
        self.channels[index].tone_off_bit = (!tone_enable) as usize;
        self.channels[index].noise_off_bit = (!noise_enable) as usize;
        self.channels[index].envelope_enabled = envelope_enabled;
    }

    fn set_volume(&mut self, index: usize, volume: usize) {
        self.channels[index].volume = volume & 0x0F;
    }

    fn set_envelope(&mut self, period: u16) {
        self.envelope_period = (period == 0) as u16 | period;
    }

    fn set_envelope_shape(&mut self, shape: usize) {
        self.envelope_shape = shape & 0x0F;
        self.envelope_counter = 0;
        self.envelope_segment = 0;
        self.reset_segment();
    }

    fn process(&mut self) {
        self.fir_index = (self.fir_index + 1) % (FIR_SIZE / DECIMATE_FACTOR - 1);
        for i in (0..DECIMATE_FACTOR).rev() {
            self.x += self.step;
            if self.x >= 1.0 {
                self.x -= 1.0;
                self.interpolator_left.y[0] = self.interpolator_left.y[1];
                self.interpolator_left.y[1] = self.interpolator_left.y[2];
                self.interpolator_left.y[2] = self.interpolator_left.y[3];
                self.interpolator_right.y[0] = self.interpolator_right.y[1];
                self.interpolator_right.y[1] = self.interpolator_right.y[2];
                self.interpolator_right.y[2] = self.interpolator_right.y[3];
                self.update_mixer();
                self.interpolator_left.y[3] = self.left;
                self.interpolator_right.y[3] = self.right;
                let y1 = self.interpolator_left.y[2] - self.interpolator_left.y[0];
                self.interpolator_left.c[0] = 0.5 * self.interpolator_left.y[1]
                    + 0.25 * (self.interpolator_left.y[0] + self.interpolator_left.y[2]);
                self.interpolator_left.c[1] = 0.5 * y1;
                self.interpolator_left.c[2] =
                    0.25 * (self.interpolator_left.y[3] - self.interpolator_left.y[1] - y1);
                let y1 = self.interpolator_right.y[2] - self.interpolator_right.y[0];
                self.interpolator_right.c[0] = 0.5 * self.interpolator_right.y[1]
                    + 0.25 * (self.interpolator_right.y[0] + self.interpolator_right.y[2]);
                self.interpolator_right.c[1] = 0.5 * y1;
                self.interpolator_right.c[2] =
                    0.25 * (self.interpolator_right.y[3] - self.interpolator_right.y[1] - y1);
            }
            self.fir_left[FIR_SIZE - self.fir_index * DECIMATE_FACTOR..][i] =
                (self.interpolator_left.c[2] * self.x + self.interpolator_left.c[1]) * self.x
                    + self.interpolator_left.c[0];
            self.fir_right[FIR_SIZE - self.fir_index * DECIMATE_FACTOR..][i] =
                (self.interpolator_right.c[2] * self.x + self.interpolator_right.c[1]) * self.x
                    + self.interpolator_right.c[0];
        }
        self.left = decimate(&mut self.fir_left[FIR_SIZE - self.fir_index * DECIMATE_FACTOR..]);
        self.right = decimate(&mut self.fir_right[FIR_SIZE - self.fir_index * DECIMATE_FACTOR..]);
    }

    fn apply_dc_filter(&mut self) {
        self.left = apply_dc_filter_for_sample(&mut self.dc_left, self.dc_index, self.left);
        self.right = apply_dc_filter_for_sample(&mut self.dc_right, self.dc_index, self.right);
        self.dc_index = (self.dc_index + 1) & (DC_FILTER_SIZE - 1);
    }

    fn slide_up(&mut self) {
        if self.envelope == 31 {
            self.envelope_segment ^= 1;
            self.reset_segment();
        } else {
            self.envelope += 1;
        }
    }

    fn slide_down(&mut self) {
        if self.envelope == 0 {
            self.envelope_segment ^= 1;
            self.reset_segment();
        } else {
            self.envelope -= 1;
        }
    }

    fn hold_top(&mut self) {}

    fn hold_bottom(&mut self) {}

    fn reset_segment(&mut self) {
        if ENVELOPE_RESET_TO_MAX[self.envelope_shape][self.envelope_segment] {
            self.envelope = 31;
            return;
        }
        self.envelope = 0;
    }

    fn update_tone(&mut self, index: usize) -> usize {
        let ch = &mut self.channels.as_mut()[index];
        ch.tone_counter += 1;
        if ch.tone_counter >= ch.tone_period {
            ch.tone_counter = 0;
            ch.tone ^= 1;
        }

        ch.tone
    }

    fn update_noise(&mut self) -> usize {
        self.noise_counter += 1;
        if self.noise_counter >= self.noise_period << 1 {
            self.noise_counter = 0;
            let bit0x3 = (self.noise ^ (self.noise >> 3)) & 1;
            self.noise = (self.noise >> 1) | (bit0x3 << 16);
        }

        self.noise & 1
    }

    fn update_envelope(&mut self) -> usize {
        self.envelope_counter += 1;
        if self.envelope_counter >= self.envelope_period {
            self.envelope_counter = 0;
            ENVELOPES[self.envelope_shape][self.envelope_segment](self);
        }
        self.envelope
    }

    fn update_mixer(&mut self) {
        let noise = self.update_noise();
        let envelope = self.update_envelope();
        self.left = 0.0;
        self.right = 0.0;
        for i in 0..TONE_CHANNELS {
            let mut out = (self.update_tone(i) | self.channels[i].tone_off_bit)
                & (noise | self.channels[i].noise_off_bit);
            out *= if self.channels[i].envelope_enabled {
                envelope
            } else {
                self.channels[i].volume * 2 + 1
            };
            assert!(out < 32);
            self.left += self.dac_table[out] * self.channels[i].pan_left;
            self.right += self.dac_table[out] * self.channels[i].pan_right;
        }
    }
}

#[allow(clippy::excessive_precision)]
fn decimate(x: &mut [f64]) -> f64 {
    assert!(x.len() >= FIR_SIZE);

    let y = -0.0000046183113992051936 * (x[1] + x[191])
        + -0.00001117761640887225 * (x[2] + x[190])
        + -0.000018610264502005432 * (x[3] + x[189])
        + -0.000025134586135631012 * (x[4] + x[188])
        + -0.000028494281690666197 * (x[5] + x[187])
        + -0.000026396828793275159 * (x[6] + x[186])
        + -0.000017094212558802156 * (x[7] + x[185])
        + 0.000023798193576966866 * (x[9] + x[183])
        + 0.000051281160242202183 * (x[10] + x[182])
        + 0.00007762197826243427 * (x[11] + x[181])
        + 0.000096759426664120416 * (x[12] + x[180])
        + 0.00010240229300393402 * (x[13] + x[179])
        + 0.000089344614218077106 * (x[14] + x[178])
        + 0.000054875700118949183 * (x[15] + x[177])
        + -0.000069839082210680165 * (x[17] + x[175])
        + -0.0001447966132360757 * (x[18] + x[174])
        + -0.00021158452917708308 * (x[19] + x[173])
        + -0.00025535069106550544 * (x[20] + x[172])
        + -0.00026228714374322104 * (x[21] + x[171])
        + -0.00022258805927027799 * (x[22] + x[170])
        + -0.00013323230495695704 * (x[23] + x[169])
        + 0.00016182578767055206 * (x[25] + x[167])
        + 0.00032846175385096581 * (x[26] + x[166])
        + 0.00047045611576184863 * (x[27] + x[165])
        + 0.00055713851457530944 * (x[28] + x[164])
        + 0.00056212565121518726 * (x[29] + x[163])
        + 0.00046901918553962478 * (x[30] + x[162])
        + 0.00027624866838952986 * (x[31] + x[161])
        + -0.00032564179486838622 * (x[33] + x[159])
        + -0.00065182310286710388 * (x[34] + x[158])
        + -0.00092127787309319298 * (x[35] + x[157])
        + -0.0010772534348943575 * (x[36] + x[156])
        + -0.0010737727700273478 * (x[37] + x[155])
        + -0.00088556645390392634 * (x[38] + x[154])
        + -0.00051581896090765534 * (x[39] + x[153])
        + 0.00059548767193795277 * (x[41] + x[151])
        + 0.0011803558710661009 * (x[42] + x[150])
        + 0.0016527320270369871 * (x[43] + x[149])
        + 0.0019152679330965555 * (x[44] + x[148])
        + 0.0018927324805381538 * (x[45] + x[147])
        + 0.0015481870327877937 * (x[46] + x[146])
        + 0.00089470695834941306 * (x[47] + x[145])
        + -0.0010178225878206125 * (x[49] + x[143])
        + -0.0020037400552054292 * (x[50] + x[142])
        + -0.0027874356824117317 * (x[51] + x[141])
        + -0.003210329988021943 * (x[52] + x[140])
        + -0.0031540624117984395 * (x[53] + x[139])
        + -0.0025657163651900345 * (x[54] + x[138])
        + -0.0014750752642111449 * (x[55] + x[137])
        + 0.0016624165446378462 * (x[57] + x[135])
        + 0.0032591192839069179 * (x[58] + x[134])
        + 0.0045165685815867747 * (x[59] + x[133])
        + 0.0051838984346123896 * (x[60] + x[132])
        + 0.0050774264697459933 * (x[61] + x[131])
        + 0.0041192521414141585 * (x[62] + x[130])
        + 0.0023628575417966491 * (x[63] + x[129])
        + -0.0026543507866759182 * (x[65] + x[127])
        + -0.0051990251084333425 * (x[66] + x[126])
        + -0.0072020238234656924 * (x[67] + x[125])
        + -0.0082672928192007358 * (x[68] + x[124])
        + -0.0081033739572956287 * (x[69] + x[123])
        + -0.006583111539570221 * (x[70] + x[122])
        + -0.0037839040415292386 * (x[71] + x[121])
        + 0.0042781252851152507 * (x[73] + x[119])
        + 0.0084176358598320178 * (x[74] + x[118])
        + 0.01172566057463055 * (x[75] + x[117])
        + 0.013550476647788672 * (x[76] + x[116])
        + 0.013388189369997496 * (x[77] + x[115])
        + 0.010979501242341259 * (x[78] + x[114])
        + 0.006381274941685413 * (x[79] + x[113])
        + -0.007421229604153888 * (x[81] + x[111])
        + -0.01486456304340213 * (x[82] + x[110])
        + -0.021143584622178104 * (x[83] + x[109])
        + -0.02504275058758609 * (x[84] + x[108])
        + -0.025473530942547201 * (x[85] + x[107])
        + -0.021627310017882196 * (x[86] + x[106])
        + -0.013104323383225543 * (x[87] + x[105])
        + 0.017065133989980476 * (x[89] + x[103])
        + 0.036978919264451952 * (x[90] + x[102])
        + 0.05823318062093958 * (x[91] + x[101])
        + 0.079072012081405949 * (x[92] + x[100])
        + 0.097675998716952317 * (x[93] + x[99])
        + 0.11236045936950932 * (x[94] + x[98])
        + 0.12176343577287731 * (x[95] + x[97])
        + 0.125 * x[96];

    let (src, dest) = x.split_at_mut(FIR_SIZE - DECIMATE_FACTOR);
    dest[0..DECIMATE_FACTOR].copy_from_slice(&src[0..DECIMATE_FACTOR]);

    y
}

fn apply_dc_filter_for_sample(dc: &mut DcFilter, index: usize, x: f64) -> f64 {
    dc.sum += -dc.delay[index] + x;
    dc.delay[index] = x;
    x - dc.sum / DC_FILTER_SIZE as f64
}

impl AymPrecise {
    /// Enabled dc filter for samples
    pub fn enable_dc_filter(&mut self) {
        self.dc_filter = true;
    }
}

impl AymBackend for AymPrecise {
    type SoundSample = f64;

    fn new(chip: SoundChip, mode: AyMode, frequency: usize, sample_rate: usize) -> Self {
        let mut ay = AymPrecise::new(matches!(chip, SoundChip::YM), frequency as f64, sample_rate);

        let (pan_a, pan_b, pan_c) = match mode {
            AyMode::Mono => (0.5, 0.5, 0.5),
            AyMode::ABC => (0.0, 0.5, 1.0),
            AyMode::ACB => (0.0, 1.0, 0.5),
            AyMode::BAC => (0.5, 0.0, 1.0),
            AyMode::BCA => (1.0, 0.0, 0.5),
            AyMode::CAB => (0.5, 1.0, 0.0),
            AyMode::CBA => (1.0, 0.5, 0.0),
        };
        ay.set_pan(0, pan_a, true);
        ay.set_pan(1, pan_b, true);
        ay.set_pan(2, pan_c, true);
        ay
    }

    fn write_register(&mut self, address: u8, value: u8) {
        if address as usize >= AY_REGISTER_COUNT {
            return;
        }

        self.registers[address as usize] = value;

        let r = self.registers;

        match address {
            0 | 1 => self.set_tone(0, u16::from_le_bytes([r[0], r[1] & 0x0f])),
            2 | 3 => self.set_tone(1, u16::from_le_bytes([r[2], r[3] & 0x0f])),
            4 | 5 => self.set_tone(2, u16::from_le_bytes([r[4], r[5] & 0x0f])),
            6 => self.set_noise((r[6] & 0x1f) as u16),
            7 => {
                self.set_mixer(
                    0,
                    (r[7] & 0x01) == 0,
                    (r[7] & 0x08) == 0,
                    (r[8] & 0x10) != 0,
                );
                self.set_mixer(
                    1,
                    (r[7] & 0x02) == 0,
                    (r[7] & 0x10) == 0,
                    (r[9] & 0x10) != 0,
                );
                self.set_mixer(
                    2,
                    (r[7] & 0x04) == 0,
                    (r[7] & 0x20) == 0,
                    (r[10] & 0x10) != 0,
                );
            }
            8 => {
                self.set_mixer(
                    0,
                    (r[7] & 0x01) == 0,
                    (r[7] & 0x08) == 0,
                    (r[8] & 0x10) != 0,
                );
                self.set_volume(0, (r[8] & 0x0F) as usize);
            }
            9 => {
                self.set_mixer(
                    1,
                    (r[7] & 0x02) == 0,
                    (r[7] & 0x10) == 0,
                    (r[9] & 0x10) != 0,
                );
                self.set_volume(1, (r[9] & 0x0F) as usize);
            }
            10 => {
                self.set_mixer(
                    2,
                    (r[7] & 0x04) == 0,
                    (r[7] & 0x20) == 0,
                    (r[10] & 0x10) != 0,
                );
                self.set_volume(2, (r[10] & 0x0F) as usize);
            }
            11 | 12 => self.set_envelope(u16::from_le_bytes([r[11], r[12]])),
            13 => self.set_envelope_shape((r[13] & 0x0F) as usize),
            _ => unreachable!(),
        }
    }

    fn next_sample(&mut self) -> StereoSample<Self::SoundSample> {
        self.process();

        if self.dc_filter {
            self.apply_dc_filter();
        }

        StereoSample {
            left: self.left,
            right: self.right,
        }
    }
}
