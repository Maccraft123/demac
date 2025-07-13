use binrw::{BinRead, BinWrite};
use strum::{Display, EnumIter};
use std::fmt;
use thiserror::Error;

#[derive(Copy, Clone, Debug, EnumIter, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub enum ScriptCode {
    #[brw(magic = 0xff_u8)]
    System,
    #[brw(magic = 0xfe_u8)]
    Current,
    #[brw(magic = 00_u8)]
    Roman,
    #[brw(magic = 01_u8)]
    Japanese,
    #[brw(magic = 02_u8)]
    TradChinese,
    #[brw(magic = 03_u8)]
    Korean,
    #[brw(magic = 04_u8)]
    Arabic,
    #[brw(magic = 05_u8)]
    Hebrew,
    #[brw(magic = 06_u8)]
    Greek,
    #[brw(magic = 07_u8)]
    Cyrillic,
    #[brw(magic = 08_u8)]
    RightToLeftSymbols,
    #[brw(magic = 09_u8)]
    Devanagari,
    #[brw(magic = 10_u8)]
    Gurmukhi,
    #[brw(magic = 11_u8)]
    Gujarati,
    #[brw(magic = 12_u8)]
    Oriya,
    #[brw(magic = 13_u8)]
    Bengali,
    #[brw(magic = 14_u8)]
    Tamil,
    #[brw(magic = 15_u8)]
    Telugu,
    #[brw(magic = 16_u8)]
    Kannada,
    #[brw(magic = 17_u8)]
    Malayalam,
    #[brw(magic = 18_u8)]
    Sinhalese,
    #[brw(magic = 19_u8)]
    Burmese,
    #[brw(magic = 20_u8)]
    Khmer,
    #[brw(magic = 21_u8)]
    Thai,
    #[brw(magic = 22_u8)]
    Laotian,
    #[brw(magic = 23_u8)]
    Georgian,
    #[brw(magic = 24_u8)]
    Armenian,
    #[brw(magic = 25_u8)]
    SimpChinese,
    #[brw(magic = 26_u8)]
    Tibetan,
    #[brw(magic = 27_u8)]
    Mongolian,
    #[brw(magic = 28_u8)]
    GeezEthiopic,
    #[brw(magic = 29_u8)]
    EastEurRoman,
    #[brw(magic = 30_u8)]
    VietnameseRoman,
    #[brw(magic = 31_u8)]
    Sindhi,
    #[brw(magic = 32_u8)]
    Uninterpreted,
}

#[derive(Copy, Clone, Debug, EnumIter, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub enum LanguageCode {
    #[brw(magic = 0_u8)]
    English,
    #[brw(magic = 1_u8)]
    French,
    #[brw(magic = 2_u8)]
    German,
    #[brw(magic = 3_u8)]
    Italian,
    #[brw(magic = 4_u8)]
    Dutch,
    #[brw(magic = 5_u8)]
    Swedish,
    #[brw(magic = 6_u8)]
    Spanish,
    #[brw(magic = 7_u8)]
    Danish,
    #[brw(magic = 8_u8)]
    Portuguese,
    #[brw(magic = 9_u8)]
    Norwegian,
    #[brw(magic = 10_u8)]
    Hebrew,
    #[brw(magic = 11_u8)]
    Japanese,
    #[brw(magic = 12_u8)]
    Arabic,
    #[brw(magic = 13_u8)]
    Finnish,
    #[brw(magic = 14_u8)]
    Greek,
    #[brw(magic = 15_u8)]
    Icelandic,
    #[brw(magic = 16_u8)]
    Maltese,
    #[brw(magic = 17_u8)]
    Turkish,
    #[brw(magic = 18_u8)]
    Croatian,
    #[brw(magic = 19_u8)]
    TradChinese,
    #[brw(magic = 20_u8)]
    Urdu,
    #[brw(magic = 21_u8)]
    Hindi,
    #[brw(magic = 22_u8)]
    Thai,
    #[brw(magic = 23_u8)]
    Korean,
    #[brw(magic = 24_u8)]
    Lithuanian,
    #[brw(magic = 25_u8)]
    Polish,
    #[brw(magic = 26_u8)]
    Hungarian,
    #[brw(magic = 27_u8)]
    Estonian,
    #[brw(magic = 28_u8)]
    Latvian,
    #[brw(magic = 29_u8)]
    Lappish,
    #[brw(magic = 30_u8)]
    Faeroese,
    #[brw(magic = 31_u8)]
    Farsi,
    #[brw(magic = 32_u8)]
    Russian,
    #[brw(magic = 33_u8)]
    SimpChinese,
    #[brw(magic = 34_u8)]
    Flemish,
    #[brw(magic = 35_u8)]
    Irish,
    #[brw(magic = 36_u8)]
    Albanian,
    #[brw(magic = 37_u8)]
    Romanian,
    #[brw(magic = 38_u8)]
    Czech,
    #[brw(magic = 39_u8)]
    Slovak,
    #[brw(magic = 40_u8)]
    Slovenian,
    #[brw(magic = 41_u8)]
    Yiddish,
    #[brw(magic = 42_u8)]
    Serbian,
    #[brw(magic = 43_u8)]
    Macedonian,
    #[brw(magic = 44_u8)]
    Bulgarian,
    #[brw(magic = 45_u8)]
    Ukrainian,
    #[brw(magic = 46_u8)]
    Byelorussian,
    #[brw(magic = 47_u8)]
    Uzbek,
    #[brw(magic = 48_u8)]
    Kazakh,
    #[brw(magic = 49_u8)]
    Azerbaijani,
    #[brw(magic = 50_u8)]
    AzerbaijaniAr,
    #[brw(magic = 51_u8)]
    Armenian,
    #[brw(magic = 52_u8)]
    Georgian,
    #[brw(magic = 53_u8)]
    Moldovan,
    #[brw(magic = 54_u8)]
    Kirghiz,
    #[brw(magic = 55_u8)]
    Tajiki,
    #[brw(magic = 56_u8)]
    Turkmen,
    #[brw(magic = 57_u8)]
    Mongolian,
    #[brw(magic = 58_u8)]
    MongolianCyr,
    #[brw(magic = 59_u8)]
    Pashto,
    #[brw(magic = 60_u8)]
    Kurdish,
    #[brw(magic = 61_u8)]
    Kashmiri,
    #[brw(magic = 62_u8)]
    Sindhi,
    #[brw(magic = 63_u8)]
    Tibetan,
    #[brw(magic = 64_u8)]
    Nepali,
    #[brw(magic = 65_u8)]
    Sanskrit,
    #[brw(magic = 66_u8)]
    Marathi,
    #[brw(magic = 67_u8)]
    Bengali,
    #[brw(magic = 68_u8)]
    Assamese,
    #[brw(magic = 69_u8)]
    Gujarati,
    #[brw(magic = 70_u8)]
    Punjabi,
    #[brw(magic = 71_u8)]
    Oriya,
    #[brw(magic = 72_u8)]
    Malayalam,
    #[brw(magic = 73_u8)]
    Kannada,
    #[brw(magic = 74_u8)]
    Tamil,
    #[brw(magic = 75_u8)]
    Telugu,
    #[brw(magic = 76_u8)]
    Sinhalese,
    #[brw(magic = 77_u8)]
    Burmese,
    #[brw(magic = 78_u8)]
    Khmer,
    #[brw(magic = 79_u8)]
    Lao,
    #[brw(magic = 80_u8)]
    Vietnamese,
    #[brw(magic = 81_u8)]
    Indonesian,
    #[brw(magic = 82_u8)]
    Tagalog,
    #[brw(magic = 83_u8)]
    MalayRoman,
    #[brw(magic = 84_u8)]
    MalayArabic,
    #[brw(magic = 85_u8)]
    Amharic,
    #[brw(magic = 86_u8)]
    Tigrinya,
    #[brw(magic = 87_u8)]
    Galla,
    #[brw(magic = 88_u8)]
    Somali,
    #[brw(magic = 89_u8)]
    Swahili,
    #[brw(magic = 90_u8)]
    Ruanda,
    #[brw(magic = 91_u8)]
    Rundi,
    #[brw(magic = 92_u8)]
    Chewa,
    #[brw(magic = 93_u8)]
    Malagasy,
    #[brw(magic = 94_u8)]
    Esperanto,
    #[brw(magic = 128_u8)]
    Welsh,
    #[brw(magic = 129_u8)]
    Basque,
    #[brw(magic = 130_u8)]
    Catalan,
    #[brw(magic = 131_u8)]
    Latin,
    #[brw(magic = 132_u8)]
    Quechua,
    #[brw(magic = 133_u8)]
    Guarani,
    #[brw(magic = 134_u8)]
    Aymara,
    #[brw(magic = 135_u8)]
    Tatar,
    #[brw(magic = 136_u8)]
    Uighur,
    #[brw(magic = 137_u8)]
    Bhutanese,
    #[brw(magic = 138_u8)]
    Javanese,
    #[brw(magic = 139_u8)]
    Sundanese,
}

#[derive(Copy, Clone, Debug, Display, EnumIter, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub enum RegionCode {
    #[brw(magic = 00_u8)]
    #[strum(to_string = "USA")]
    UnitedStates,
    #[brw(magic = 01_u8)]
    France,
    #[brw(magic = 02_u8)]
    Britain,
    #[brw(magic = 03_u8)]
    Germany,
    #[brw(magic = 04_u8)]
    Italy,
    #[brw(magic = 05_u8)]
    Netherlands,
    #[brw(magic = 06_u8)]
    #[strum(to_string = "Belgium Lux.")]
    FrBelgiumLux,
    #[brw(magic = 07_u8)]
    Sweden,
    #[brw(magic = 09_u8)]
    Denmark,
    #[brw(magic = 10_u8)]
    Portugal,
    #[brw(magic = 11_u8)]
    #[strum(to_string = "Fr. Canada")]
    FrCanada,
    #[brw(magic = 12_u8)]
    Norway,
    #[brw(magic = 13_u8)]
    Israel,
    #[brw(magic = 14_u8)]
    Japan,
    #[brw(magic = 15_u8)]
    Australia,
    #[brw(magic = 16_u8)]
    Arabia,
    #[brw(magic = 17_u8)]
    Finland,
    #[brw(magic = 18_u8)]
    #[strum(to_string = "Fr. Swiss.")]
    FrSwiss,
    #[brw(magic = 19_u8)]
    #[strum(to_string = "Gr. Swiss.")]
    GrSwiss,
    #[brw(magic = 20_u8)]
    Greece,
    #[brw(magic = 21_u8)]
    Iceland,
    #[brw(magic = 22_u8)]
    Malta,
    #[brw(magic = 23_u8)]
    Cyprus,
    #[brw(magic = 24_u8)]
    Turkey,
    #[brw(magic = 25_u8)]
    #[strum(to_string = "Yugoslavia")]
    YugoCroatian,
    #[brw(magic = 33_u8)]
    #[strum(to_string = "India")]
    IndiaHindi,
    #[brw(magic = 34_u8)]
    Pakistan,
    #[brw(magic = 36_u8)]
    #[strum(to_string = "It. Swiss.")]
    ItSwiss,
    //#[strum(to_string = "Anc. Greek")]
    //#[brw(magic = 41_u8)]
    //AncGreek,
    #[brw(magic = 41_u8)]
    Lithuania,
    #[brw(magic = 42_u8)]
    Poland,
    #[brw(magic = 43_u8)]
    Hungary,
    #[brw(magic = 44_u8)]
    Estonia,
    #[brw(magic = 45_u8)]
    Latvia,
    #[brw(magic = 46_u8)]
    Lapland,
    #[brw(magic = 47_u8)]
    #[strum(to_string = "Faeroe Isl.")]
    FaeroeIsl,
    #[brw(magic = 48_u8)]
    Iran,
    #[brw(magic = 49_u8)]
    Russia,
    #[brw(magic = 50_u8)]
    Ireland,
    #[brw(magic = 51_u8)]
    Korea,
    #[brw(magic = 52_u8)]
    China,
    #[brw(magic = 53_u8)]
    Taiwan,
    #[brw(magic = 54_u8)]
    Thailand,
}

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum ScriptError {
    #[error("Character {0} does not exist in {0}")]
    InvalidChar(char, ScriptCode),
}

pub trait MacScript: fmt::Debug + Copy + Clone + From<u8> + Into<u8> + TryFrom<char> + Into<char> {
    const CODE: ScriptCode;
    fn decode(v: u8) -> char {
        Self::from(v).into()
    }
    fn encode(ch: char) -> Result<u8, <Self as TryFrom<char>>::Error> {
        Self::try_from(ch).map(|v| v.into())
    }
    fn to_char(self) -> char {
        self.into()
    }
    fn to_u8(self) -> u8 {
        self.into()
    }
}

macro_rules! script {
    ($name: ident, $code: expr, $(($idx: literal => $unicode: literal)),*) => {
        #[derive(BinRead, BinWrite, Hash, Copy, Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct $name(u8);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.to_char())
            }
        }

        impl MacScript for $name {
            const CODE: ScriptCode = $code;
        }

        impl TryFrom<char> for $name {
            type Error = ScriptError;
            fn try_from(ch: char) -> Result<$name, ScriptError> {
                match ch {
                    $($unicode => Ok(Self($idx)),)*
                    _ => Err(ScriptError::InvalidChar(ch, Self::CODE)),
                }
            }
        }

        impl From<$name> for char {
            fn from(f: $name) -> char {
                match f.0 {
                    0..128 => f.0 as char,
                    $($idx => $unicode,)*
                }
            }
        }

        impl From<u8> for $name {
            fn from(f: u8) -> $name {
                Self(f)
            }
        }

        impl From<$name> for u8 {
            fn from(f: $name) -> u8 {
                f.0
            }
        }
   }
}

script!(MacRoman, ScriptCode::Roman,
    (0x80 => 'Ä'),
    (0x81 => 'Å'),
    (0x82 => 'Ç'),
    (0x83 => 'É'),
    (0x84 => 'Ñ'),
    (0x85 => 'Ö'),
    (0x86 => 'Ü'),
    (0x87 => 'á'),
    (0x88 => 'à'),
    (0x89 => 'â'),
    (0x8a => 'ä'),
    (0x8b => 'ã'),
    (0x8c => 'å'),
    (0x8d => 'ç'),
    (0x8e => 'é'),
    (0x8f => 'è'),
    (0x90 => 'ê'),
    (0x91 => 'ë'),
    (0x92 => 'í'),
    (0x93 => 'ì'),
    (0x94 => 'î'),
    (0x95 => 'ï'),
    (0x96 => 'ñ'),
    (0x97 => 'ó'),
    (0x98 => 'ò'),
    (0x99 => 'ô'),
    (0x9a => 'ö'),
    (0x9b => 'õ'),
    (0x9c => 'ú'),
    (0x9d => 'ù'),
    (0x9e => 'û'),
    (0x9f => 'ü'),
    (0xa0 => '†'),
    (0xa1 => '°'),
    (0xa2 => '¢'),
    (0xa3 => '£'),
    (0xa4 => '§'),
    (0xa5 => '•'),
    (0xa6 => '¶'),
    (0xa7 => 'ß'),
    (0xa8 => '®'),
    (0xa9 => '©'),
    (0xaa => '™'),
    (0xab => '´'),
    (0xac => '¨'),
    (0xad => '≠'),
    (0xae => 'Æ'),
    (0xaf => 'Ø'),
    (0xb0 => '∞'),
    (0xb1 => '±'),
    (0xb2 => '≤'),
    (0xb3 => '≥'),
    (0xb4 => '¥'),
    (0xb5 => 'µ'),
    (0xb6 => '∂'),
    (0xb7 => '∑'),
    (0xb8 => '∏'),
    (0xb9 => 'π'),
    (0xba => '∫'),
    (0xbb => 'ª'),
    (0xbc => 'º'),
    (0xbd => 'Ω'),
    (0xbe => 'æ'),
    (0xbf => 'ø'),
    (0xc0 => '¿'),
    (0xc1 => '¡'),
    (0xc2 => '¬'),
    (0xc3 => '√'),
    (0xc4 => 'ƒ'),
    (0xc5 => '≈'),
    (0xc6 => '∆'),
    (0xc7 => '«'),
    (0xc8 => '»'),
    (0xc9 => '…'),
    (0xca => '\u{a0}'), // nbsp
    (0xcb => 'À'),
    (0xcc => 'Ã'),
    (0xcd => 'Õ'),
    (0xce => 'Œ'),
    (0xcf => 'œ'),
    (0xd0 => '–'),
    (0xd1 => '—'),
    (0xd2 => '“'),
    (0xd3 => '”'),
    (0xd4 => '‘'),
    (0xd5 => '’'),
    (0xd6 => '÷'),
    (0xd7 => '◊'),
    (0xd8 => 'ÿ'),
    (0xd9 => 'Ÿ'),
    (0xda => '⁄'),
    (0xdb => '€'),
    (0xdc => '‹'),
    (0xdd => '›'),
    (0xde => 'ﬁ'),
    (0xdf => 'ﬂ'),
    (0xe0 => '‡'),
    (0xe1 => '·'),
    (0xe2 => '‚'),
    (0xe3 => '„'),
    (0xe4 => '‰'),
    (0xe5 => 'Â'),
    (0xe6 => 'Ê'),
    (0xe7 => 'Á'),
    (0xe8 => 'Ë'),
    (0xe9 => 'È'),
    (0xea => 'Í'),
    (0xeb => 'Î'),
    (0xec => 'Ï'),
    (0xed => 'Ì'),
    (0xee => 'Ó'),
    (0xef => 'Ô'),
    //(0xf0 => '🍎'),
    (0xf0 => '\u{f8ff}'), // apple symbol
    (0xf1 => 'Ò'),
    (0xf2 => 'Ú'),
    (0xf3 => 'Û'),
    (0xf4 => 'Ù'),
    (0xf5 => 'ı'),
    (0xf6 => 'ˆ'),
    (0xf7 => '˜'),
    (0xf8 => '¯'),
    (0xf9 => '˘'),
    (0xfa => '˙'),
    (0xfb => '˚'),
    (0xfc => '¸'),
    (0xfd => '˝'),
    (0xfe => '˛'),
    (0xff => 'ˇ')
);

/*pub fn macroman_encode(ch: char) -> Option<u8> {
    MACROMAN_TO_CHAR
        .iter()
        .enumerate()
        .find_map(|(i, c)| (*c == ch).then_some(i as u8))
}

pub fn macroman_decode(ch: u8) -> char {
    MACROMAN_TO_CHAR[ch as usize]
}

static MACROMAN_TO_CHAR: [char; 256] = [
    '\0', '\u{1}', '\u{2}', '\u{3}', '\u{4}', '\u{5}', '\u{6}', '\u{7}', '\u{8}', '\t', '\n',
    '\u{b}', '\u{c}', '\r', '\u{e}', '\u{f}', '\u{10}', '\u{11}', '\u{12}', '\u{13}', '\u{14}',
    '\u{15}', '\u{16}', '\u{17}', '\u{18}', '\u{19}', '\u{1a}', '\u{1b}', '\u{1c}', '\u{1d}',
    '\u{1e}', '\u{1f}', ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.',
    '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A',
    'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    '{', '|', '}', '~', '\u{7f}', 'Ä', 'Å', 'Ç', 'É', 'Ñ', 'Ö', 'Ü', 'á', 'à', 'â', 'ä', 'ã', 'å',
    'ç', 'é', 'è', 'ê', 'ë', 'í', 'ì', 'î', 'ï', 'ñ', 'ó', 'ò', 'ô', 'ö', 'õ', 'ú', 'ù', 'û', 'ü',
    '†', '°', '¢', '£', '§', '•', '¶', 'ß', '®', '©', '™', '´', '¨', '≠', 'Æ', 'Ø', '∞', '±', '≤',
    '≥', '¥', 'µ', '∂', '∑', '∏', 'π', '∫', 'ª', 'º', 'Ω', 'æ', 'ø', '¿', '¡', '¬', '√', 'ƒ', '≈',
    '∆', '«', '»', '…', '\u{a0}', 'À', 'Ã', 'Õ', 'Œ', 'œ', '–', '—', '“', '”', '‘', '’', '÷', '◊',
    'ÿ', 'Ÿ', '⁄', '€', '‹', '›', 'ﬁ', 'ﬂ', '‡', '·', '‚', '„', '‰', 'Â', 'Ê', 'Á', 'Ë', 'È', 'Í',
    'Î', 'Ï', 'Ì', 'Ó', 'Ô', '\u{f8ff}', 'Ò', 'Ú', 'Û', 'Ù', 'ı', 'ˆ', '˜', '¯', '˘', '˙', '˚',
    '¸', '˝', '˛', 'ˇ',
];*/
