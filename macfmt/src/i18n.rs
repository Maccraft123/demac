use binrw::{BinRead, BinWrite};

#[derive(Copy, Clone, Debug, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub enum ScriptCode {
    #[brw(magic = 0xff_u8)] System,
    #[brw(magic = 0xfe_u8)] Current,
    #[brw(magic = 00_u8)] Roman,
    #[brw(magic = 01_u8)] Japanese,
    #[brw(magic = 02_u8)] TradChinese,
    #[brw(magic = 03_u8)] Korean,
    #[brw(magic = 04_u8)] Arabic,
    #[brw(magic = 05_u8)] Hebrew,
    #[brw(magic = 06_u8)] Greek,
    #[brw(magic = 07_u8)] Cyrillic,
    #[brw(magic = 08_u8)] RightToLeftSymbols,
    #[brw(magic = 09_u8)] Devanagari,
    #[brw(magic = 10_u8)] Gurmukhi,
    #[brw(magic = 11_u8)] Gujarati,
    #[brw(magic = 12_u8)] Oriya,
    #[brw(magic = 13_u8)] Bengali,
    #[brw(magic = 14_u8)] Tamil,
    #[brw(magic = 15_u8)] Telugu,
    #[brw(magic = 16_u8)] Kannada,
    #[brw(magic = 17_u8)] Malayalam,
    #[brw(magic = 18_u8)] Sinhalese,
    #[brw(magic = 19_u8)] Burmese,
    #[brw(magic = 20_u8)] Khmer,
    #[brw(magic = 21_u8)] Thai,
    #[brw(magic = 22_u8)] Laotian,
    #[brw(magic = 23_u8)] Georgian,
    #[brw(magic = 24_u8)] Armenian,
    #[brw(magic = 25_u8)] SimpChinese,
    #[brw(magic = 26_u8)] Tibetan,
    #[brw(magic = 27_u8)] Mongolian,
    #[brw(magic = 28_u8)] GeezEthiopic,
    #[brw(magic = 29_u8)] EastEurRoman,
    #[brw(magic = 30_u8)] VietnameseRoman,
    #[brw(magic = 31_u8)] Sindhi,
    #[brw(magic = 32_u8)] Uninterpreted,
}

#[derive(Copy, Clone, Debug, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub enum LanguageCode {
    #[brw(magic = 0_u8)] English,
    #[brw(magic = 1_u8)] French,
    #[brw(magic = 2_u8)] German,
    #[brw(magic = 3_u8)] Italian,
    #[brw(magic = 4_u8)] Dutch,
    #[brw(magic = 5_u8)] Swedish,
    #[brw(magic = 6_u8)] Spanish,
    #[brw(magic = 7_u8)] Danish,
    #[brw(magic = 8_u8)] Portuguese,
    #[brw(magic = 9_u8)] Norwegian,
    #[brw(magic = 10_u8)] Hebrew,
    #[brw(magic = 11_u8)] Japanese,
    #[brw(magic = 12_u8)] Arabic,
    #[brw(magic = 13_u8)] Finnish,
    #[brw(magic = 14_u8)] Greek,
    #[brw(magic = 15_u8)] Icelandic,
    #[brw(magic = 16_u8)] Maltese,
    #[brw(magic = 17_u8)] Turkish,
    #[brw(magic = 18_u8)] Croatian,
    #[brw(magic = 19_u8)] TradChinese,
    #[brw(magic = 20_u8)] Urdu,
    #[brw(magic = 21_u8)] Hindi,
    #[brw(magic = 22_u8)] Thai,
    #[brw(magic = 23_u8)] Korean,
    #[brw(magic = 24_u8)] Lithuanian,
    #[brw(magic = 25_u8)] Polish,
    #[brw(magic = 26_u8)] Hungarian,
    #[brw(magic = 27_u8)] Estonian,
    #[brw(magic = 28_u8)] Latvian,
    #[brw(magic = 29_u8)] Lappish,
    #[brw(magic = 30_u8)] Faeroese,
    #[brw(magic = 31_u8)] Farsi,
    #[brw(magic = 32_u8)] Russian,
    #[brw(magic = 33_u8)] SimpChinese,
    #[brw(magic = 34_u8)] Flemish,
    #[brw(magic = 35_u8)] Irish,
    #[brw(magic = 36_u8)] Albanian,
    #[brw(magic = 37_u8)] Romanian,
    #[brw(magic = 38_u8)] Czech,
    #[brw(magic = 39_u8)] Slovak,
    #[brw(magic = 40_u8)] Slovenian,
    #[brw(magic = 41_u8)] Yiddish,
    #[brw(magic = 42_u8)] Serbian,
    #[brw(magic = 43_u8)] Macedonian,
    #[brw(magic = 44_u8)] Bulgarian,
    #[brw(magic = 45_u8)] Ukrainian,
    #[brw(magic = 46_u8)] Byelorussian,
    #[brw(magic = 47_u8)] Uzbek,
    #[brw(magic = 48_u8)] Kazakh,
    #[brw(magic = 49_u8)] Azerbaijani,
    #[brw(magic = 50_u8)] AzerbaijaniAr,
    #[brw(magic = 51_u8)] Armenian,
    #[brw(magic = 52_u8)] Georgian,
    #[brw(magic = 53_u8)] Moldovan,
    #[brw(magic = 54_u8)] Kirghiz,
    #[brw(magic = 55_u8)] Tajiki,
    #[brw(magic = 56_u8)] Turkmen,
    #[brw(magic = 57_u8)] Mongolian,
    #[brw(magic = 58_u8)] MongolianCyr,
    #[brw(magic = 59_u8)] Pashto,
    #[brw(magic = 60_u8)] Kurdish,
    #[brw(magic = 61_u8)] Kashmiri,
    #[brw(magic = 62_u8)] Sindhi,
    #[brw(magic = 63_u8)] Tibetan,
    #[brw(magic = 64_u8)] Nepali,
    #[brw(magic = 65_u8)] Sanskrit,
    #[brw(magic = 66_u8)] Marathi,
    #[brw(magic = 67_u8)] Bengali,
    #[brw(magic = 68_u8)] Assamese,
    #[brw(magic = 69_u8)] Gujarati,
    #[brw(magic = 70_u8)] Punjabi,
    #[brw(magic = 71_u8)] Oriya,
    #[brw(magic = 72_u8)] Malayalam,
    #[brw(magic = 73_u8)] Kannada,
    #[brw(magic = 74_u8)] Tamil,
    #[brw(magic = 75_u8)] Telugu,
    #[brw(magic = 76_u8)] Sinhalese,
    #[brw(magic = 77_u8)] Burmese,
    #[brw(magic = 78_u8)] Khmer,
    #[brw(magic = 79_u8)] Lao,
    #[brw(magic = 80_u8)] Vietnamese,
    #[brw(magic = 81_u8)] Indonesian,
    #[brw(magic = 82_u8)] Tagalog,
    #[brw(magic = 83_u8)] MalayRoman,
    #[brw(magic = 84_u8)] MalayArabic,
    #[brw(magic = 85_u8)] Amharic,
    #[brw(magic = 86_u8)] Tigrinya,
    #[brw(magic = 87_u8)] Galla,
    #[brw(magic = 88_u8)] Somali,
    #[brw(magic = 89_u8)] Swahili,
    #[brw(magic = 90_u8)] Ruanda,
    #[brw(magic = 91_u8)] Rundi,
    #[brw(magic = 92_u8)] Chewa,
    #[brw(magic = 93_u8)] Malagasy,
    #[brw(magic = 94_u8)] Esperanto,
    #[brw(magic = 128_u8)] Welsh,
    #[brw(magic = 129_u8)] Basque,
    #[brw(magic = 130_u8)] Catalan,
    #[brw(magic = 131_u8)] Latin,
    #[brw(magic = 132_u8)] Quechua,
    #[brw(magic = 133_u8)] Guarani,
    #[brw(magic = 134_u8)] Aymara,
    #[brw(magic = 135_u8)] Tatar,
    #[brw(magic = 136_u8)] Uighur,
    #[brw(magic = 137_u8)] Bhutanese,
    #[brw(magic = 138_u8)] Javanese,
    #[brw(magic = 139_u8)] Sundanese,
}

#[derive(Copy, Clone, Debug, BinRead, BinWrite, Eq, PartialEq)]
#[brw(big)]
pub enum RegionCode {
    #[brw(magic = 00_u8)] UnitedStates,
    #[brw(magic = 01_u8)] France,
    #[brw(magic = 02_u8)] Britain,
    #[brw(magic = 03_u8)] Germany,
    #[brw(magic = 04_u8)] Italy,
    #[brw(magic = 05_u8)] Netherlands,
    #[brw(magic = 06_u8)] FrBelgiumLux,
    #[brw(magic = 07_u8)] Sweden,
    #[brw(magic = 09_u8)] Denmark,
    #[brw(magic = 10_u8)] Portugal,
    #[brw(magic = 11_u8)] FrCanada,
    #[brw(magic = 13_u8)] Israel,
    #[brw(magic = 14_u8)] Japan,
    #[brw(magic = 15_u8)] Australia,
    #[brw(magic = 16_u8)] Arabia,
    #[brw(magic = 17_u8)] Finland,
    #[brw(magic = 18_u8)] FrSwiss,
    #[brw(magic = 19_u8)] GrSwiss,
    #[brw(magic = 20_u8)] Greece,
    #[brw(magic = 21_u8)] Iceland,
    #[brw(magic = 22_u8)] Malta,
    #[brw(magic = 23_u8)] Cyprus,
    #[brw(magic = 24_u8)] Turkey,
    #[brw(magic = 25_u8)] YugoCroatian,
    #[brw(magic = 33_u8)] IndiaHindi,
    #[brw(magic = 34_u8)] Pakistan,
    #[brw(magic = 41_u8)] Lithuania,
    #[brw(magic = 42_u8)] Poland,
    #[brw(magic = 43_u8)] Hungary,
    #[brw(magic = 44_u8)] Estonia,
    #[brw(magic = 45_u8)] Latvia,
    #[brw(magic = 46_u8)] Lapland,
    #[brw(magic = 47_u8)] FaeroeIsl,
    #[brw(magic = 48_u8)] Iran,
    #[brw(magic = 49_u8)] Russia,
    #[brw(magic = 50_u8)] Ireland,
    #[brw(magic = 51_u8)] Korea,
    #[brw(magic = 52_u8)] China,
    #[brw(magic = 53_u8)] Taiwan,
    #[brw(magic = 54_u8)] Thailand,
}
