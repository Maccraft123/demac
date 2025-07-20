use binrw::io::SeekFrom;
use binrw::{BinRead, BinResult, BinWrite};
use bitflags::bitflags;
use bitvec::field::BitField;
use bitvec::view::BitView;
use super::ResourceType;
use crate::common::{DateTime, DynamicPascalString, Point, Rect, SizedString, Style};
use crate::i18n::{RegionCode, MacRoman, MacScript};
use bitvec::order::Msb0;
use derivative::Derivative;
use std::num::NonZeroU8;
use strum::{Display, EnumIter, FromRepr};

mod lut;

const fn image_size(side: usize, bpp: usize) -> usize {
    let pixels_per_bit: usize = 8 / bpp;
    (side * side) / pixels_per_bit
}

#[derive(Clone, Derivative, Eq, PartialEq)]
#[derivative(Debug)]
pub enum Type {
    Menu(Menu),
    SystemVersion(DynamicPascalString),
    String(DynamicPascalString),
    KeyboardName(DynamicPascalString),
    StringList(StringList),
    RomOverride(RomOverride),
    MfsFolder(MfsFolder),
    Window(Window),
    Alert(Alert),
    Dialog(Dialog),
    Size(Size),
    SystemFonts(SystemFonts),
    Font(Font),
    FinderIcon(IconList<{ image_size(32, 1) }>),
    SmallIcon(IconList<{ image_size(16, 1) }>),
    SmallIcons(Icon<{ image_size(16, 1) }>),
    Icon(Icon<{ image_size(32, 1) }>),
    Pattern(Icon<{ image_size(8, 1) }>),
    LargeColorIcon4(ColorIcon<{ image_size(32, 4) }, 4>),
    LargeColorIcon8(ColorIcon<{ image_size(32, 8) }, 8>),
    SmallColorIcon4(ColorIcon<{ image_size(16, 4) }, 4>),
    SmallColorIcon8(ColorIcon<{ image_size(16, 8) }, 8>),
    FileReference(FileReference),
    ItemList(ItemList),
    Version(Version),
    Cursor(Cursor),
    Code0(Code0),
    ColorLut(ColorLut),
    Bundle(Bundle),
    Template(Template),
    Other(#[derivative(Debug = "ignore")] Vec<u8>),
}

impl Type {
    pub fn new(kind: &ResourceType, id: i16, data: Vec<u8>) -> BinResult<Type> {
        let len = data.len();
        let mut cursor = std::io::Cursor::new(data);
        Ok(match kind {
            ResourceType::SystemFontIds => Type::SystemFonts(SystemFonts::read(&mut cursor)?),
            ResourceType::Menu => Type::Menu(Menu::read(&mut cursor)?),
            ResourceType::Code if id == 0 => Type::Code0(Code0::read(&mut cursor)?),
            ResourceType::RomResourceOverrideList => {
                Type::RomOverride(RomOverride::read(&mut cursor)?)
            }
            ResourceType::String => Type::String(DynamicPascalString::read(&mut cursor)?),
            /*ResourceType::SystemVersion => {
                Type::SystemVersion(DynamicPascalString::read(&mut cursor)?)
            }*/
            ResourceType::StringList => Type::StringList(StringList::read(&mut cursor)?),
            ResourceType::MfsFolderInfo => Type::MfsFolder(MfsFolder::read(&mut cursor)?),
            ResourceType::Size => Type::Size(Size::read(&mut cursor)?),
            ResourceType::WindowTemplate => Type::Window(Window::read(&mut cursor)?),
            ResourceType::FinderIcon => Type::FinderIcon(IconList::read(&mut cursor)?),
            ResourceType::SmallIconList => Type::SmallIcon(IconList::read(&mut cursor)?),
            ResourceType::SmallIcons => Type::SmallIcons(Icon::read(&mut cursor)?),
            ResourceType::AlertBoxTemplate => Type::Alert(Alert::read(&mut cursor)?),
            ResourceType::DialogBoxTemplate => Type::Dialog(Dialog::read(&mut cursor)?),
            ResourceType::Icon => Type::Icon(Icon::read(&mut cursor)?),
            ResourceType::LargeColorIcon4 => Type::LargeColorIcon4(ColorIcon::read(&mut cursor)?),
            ResourceType::LargeColorIcon8 => Type::LargeColorIcon8(ColorIcon::read(&mut cursor)?),
            ResourceType::SmallColorIcon4 => Type::SmallColorIcon4(ColorIcon::read(&mut cursor)?),
            ResourceType::SmallColorIcon8 => Type::SmallColorIcon8(ColorIcon::read(&mut cursor)?),
            ResourceType::Icon32 => Type::Icon(Icon::read(&mut cursor)?),
            ResourceType::Pattern => Type::Pattern(Icon::read(&mut cursor)?),
            ResourceType::Cursor => Type::Cursor(Cursor::read(&mut cursor)?),
            ResourceType::Template => Type::Template(Template::read(&mut cursor)?),
            ResourceType::Bundle => Type::Bundle(Bundle::read(&mut cursor)?),
            ResourceType::BitmapFont if len > 0 => Type::Font(Font::read(&mut cursor)?),
            ResourceType::Rom128kFont if len > 0 => Type::Font(Font::read(&mut cursor)?),
            ResourceType::ItemList => Type::ItemList(ItemList::read(&mut cursor)?),
            ResourceType::VersionNumber => Type::Version(Version::read(&mut cursor)?),
            ResourceType::ColorLut => Type::ColorLut(ColorLut::read(&mut cursor)?),
            ResourceType::KeyboardName => Type::KeyboardName(DynamicPascalString::read(&mut cursor)?),
            ResourceType::FileReference => Type::FileReference(FileReference::read(&mut cursor)?),
            _ => Type::Other(cursor.into_inner()),
        })
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ColorLut {
    seed: SizedString<4>,
    flags: u16,
    size: u16,
    #[br(count = size+1)]
    entries: Vec<ClutEntry>,
}

impl ColorLut {
    pub fn entries(&self) -> &[ClutEntry] {
        &self.entries
    }
    pub fn entries_mut(&mut self) -> &mut Vec<ClutEntry> {
        &mut self.entries
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ClutEntry {
    pixel: u16,
    rgb: [u16; 3],
}

impl ClutEntry {
    pub fn pixel(&self) -> u16 {
        self.pixel
    }
    pub fn r(&self) -> u16 {
        self.rgb[0]
    }
    pub fn g(&self) -> u16 {
        self.rgb[1]
    }
    pub fn b(&self) -> u16 {
        self.rgb[2]
    }
    pub fn set_rgb_f32(&mut self, v: [f32; 3]) {
        self.rgb[0] = (v[0] * 65535.0) as u16;
        self.rgb[1] = (v[1] * 65535.0) as u16;
        self.rgb[2] = (v[2] * 65535.0) as u16;
    }
    pub fn rgb_f32(&self) -> [f32; 3] {
        [
            self.rgb[0] as f32 / 65535.0,
            self.rgb[1] as f32 / 65535.0,
            self.rgb[2] as f32 / 65535.0,
        ]
    }
    pub fn rgb(&self) -> image::Rgb<u16> {
        image::Rgb(self.rgb)
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Template {
    #[br(parse_with = binrw::helpers::until_eof)]
    fields: Vec<Field>,
}

impl Template {
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Field {
    name: DynamicPascalString,
    ty: FieldType,
}

impl Field {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn ty(&self) -> &FieldType {
        &self.ty
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum FieldType {
    // integers
    #[brw(magic = b"DBYT")]
    SignedDecimalByte,
    #[brw(magic = b"DWRD")]
    SignedDecimalWord,
    #[brw(magic = b"DLNG")]
    SignedDecimalLong,
    #[brw(magic = b"HBYT")]
    UnsignedHexByte,
    #[brw(magic = b"HWRD")]
    UnsignedHexWord,
    #[brw(magic = b"HLNG")]
    UnsignedHexLong,
    // bit and bitfields
    #[brw(magic = b"BBIT")]
    BitflagByte,
    // #[brw(magic = b"")] BitfieldByte(u16),
    // misc graphics and system
    #[brw(magic = b"BOOL")]
    BoolWord,
    #[brw(magic = b"CHAR")]
    AsciiChar,
    #[brw(magic = b"TNAM")]
    TypeName,
    #[brw(magic = b"PNT ")]
    QuickDrawPoint,
    #[brw(magic = b"RECT")]
    QuickDrawRect,
    // ascii text
    #[brw(magic = b"PSTR")]
    PascalString,
    #[brw(magic = b"ESTR")]
    EvenPaddedPascalString,
    #[brw(magic = b"OSTR")]
    OddPaddedPascalString,
    #[brw(magic = b"CSTR")]
    CString,
    #[brw(magic = b"ECST")]
    EvenPaddedCString,
    #[brw(magic = b"OCST")]
    OddPaddedCString,
    #[brw(magic = b"WSTR")]
    WordLengthString,
    #[brw(magic = b"LSTR")]
    LongLengthString,
    //#[brw(magic = b"")] PaddedPascalString(u32),
    //#[brw(magic = b"")] PaddedCString(u32),
    // hexdump
    //#[brw(magic = b"")] FixedLengthHexDump(u32),
    #[brw(magic = b"HEXD")]
    HexDump,
    // arrays
    #[brw(magic = b"OCNT")]
    OneBasedCount,
    #[brw(magic = b"ZCNT")]
    ZeroBasedCount,
    #[brw(magic = b"LSTC")]
    BeginCountedListItem,
    #[brw(magic = b"LSTB")]
    BeginNonCountedListItem,
    #[brw(magic = b"LSTZ")]
    BeginListItemNullTerminated,
    #[brw(magic = b"LSTE")]
    EndListItem,
    // alignment
    #[brw(magic = b"AWRD")]
    AlingToWord,
    #[brw(magic = b"ALNG")]
    AlignToLong,
    // filler
    #[brw(magic = b"FBYT")]
    FillByte,
    #[brw(magic = b"FWRD")]
    FillWord,
    #[brw(magic = b"FLNG")]
    FillLong,
    Unknown(SizedString<4>),
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct SystemFonts {
    count: u16,
    #[br(count = count+1)]
    ids: Vec<i16>,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Font {
    font_type: u16,
    first_char: u16,
    last_char: u16,
    wid_max: u16,
    kern_max: u16,
    ndescent: u16,
    f_rect_width: u16,
    f_rect_height: u16,
    owt_loc: u16,
    ascent: u16,
    descent: u16,
    leading: u16,
    row_words: u16,
    #[br(count = 2 * row_words * f_rect_height)]
    bit_img: Vec<u8>,
    #[br(count = 2 * (last_char - first_char + 3))]
    location_table: Vec<u8>,
    #[br(try, count = 2 * (last_char - first_char + 3))]
    offset_width_table: Option<Vec<u8>>,
    #[br(try, count = 2 * (last_char - first_char + 3))]
    char_width: Option<Vec<u8>>,
    #[br(try, count = 2 * (last_char - first_char + 3))]
    image_height: Option<Vec<u8>>,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Bundle {
    sig: SizedString<4>,
    version_res_id: i16,
    resource_type_count_minus_one: u16,
    #[br(count = resource_type_count_minus_one + 1)]
    resources: Vec<BundleResType>,
}

impl Bundle {
    pub fn sig(&self) -> &str {
        self.sig.try_as_str().unwrap()
    }
    pub fn types(&self) -> &[BundleResType] {
        &self.resources
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct BundleResType {
    res_type: SizedString<4>,
    res_count_minus_one: u16,
    #[br(count = res_count_minus_one + 1)]
    map: Vec<BundleResMap>,
}

impl BundleResType {
    pub fn type_name(&self) -> &str {
        self.res_type.try_as_str().unwrap()
    }
    pub fn res_map(&self) -> &[BundleResMap] {
        &self.map
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct BundleResMap {
    pub local: i16,
    pub actual: i16,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Code0 {
    above_a5_size: u32,
    below_a5_size: u32,
    jump_table_size: u32,
    jump_table_offset: u32,
    #[br(count = jump_table_size/8)]
    jump_table: Vec<JumpEntry>,
}

impl Code0 {
    pub fn below_a5_size(&self) -> u32 {
        self.below_a5_size
    }
    pub fn above_a5_size(&self) -> u32 {
        self.above_a5_size
    }
    pub fn entries(&self) -> &[JumpEntry] {
        &self.jump_table
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct JumpEntry {
    routine_offset: u16,
    load_segment_number: u32,
    #[br(assert(loadseg == 0xa9f0))]
    loadseg: u16,
}

impl JumpEntry {
    pub fn routine_offset(&self) -> u16 {
        self.routine_offset
    }
    pub fn load_segment_number(&self) -> u32 {
        self.load_segment_number
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct StringList {
    length: u16,
    #[br(count = length)]
    list: Vec<DynamicPascalString>,
}

impl StringList {
    pub fn list(&self) -> &[DynamicPascalString] {
        &self.list
    }
    pub fn list_mut(&mut self) -> &mut [DynamicPascalString] {
        &mut self.list
    }
}

fn from_bcd(v: u8) -> u8 {
    let hi = (v & 0xf0) >> 4;
    let lo = v & 0x0f;
    hi * 10 + lo
}

fn to_bcd(v: &u8) -> u8 {
    if *v & 0x0f > 9 { *v + 6 } else { *v }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Version {
    #[br(map = from_bcd)]
    #[bw(map = to_bcd)]
    major: u8,
    minor: u8,
    development_stage: DevelopmentStage,
    prerelease_revision: u8,
    #[brw(pad_size_to = 2)]
    region: RegionCode,
    version_number: DynamicPascalString,
    version_message: DynamicPascalString,
}

impl Version {
    pub fn version_string_short_mut(&mut self) -> &mut String {
        self.version_number.as_mut()
    }
    pub fn version_string_long_mut(&mut self) -> &mut String {
        self.version_message.as_mut()
    }
    pub fn region_code_mut(&mut self) -> &mut RegionCode {
        &mut self.region
    }
    pub fn region_code(&mut self) -> RegionCode {
        self.region
    }
    pub fn major_mut(&mut self) -> &mut u8 {
        &mut self.major
    }
    pub fn minor(&self) -> u8 {
        self.minor
    }
    pub fn major(&self) -> u8 {
        self.major
    }
    pub fn set_minor(&mut self, minor: u8) {
        self.minor = minor;
    }
    pub fn prerelease_mut(&mut self) -> &mut u8 {
        &mut self.prerelease_revision
    }
    pub fn development_stage(&self) -> DevelopmentStage {
        self.development_stage
    }
    pub fn development_stage_mut(&mut self) -> &mut DevelopmentStage {
        &mut self.development_stage
    }
}

#[derive(Copy, Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum DevelopmentStage {
    #[brw(magic = 0x20_u8)]
    PreAlpha,
    #[brw(magic = 0x40_u8)]
    Alpha,
    #[brw(magic = 0x60_u8)]
    Beta,
    #[brw(magic = 0x80_u8)]
    Released,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct FileReference {
    ty: SizedString<4>,
    icon_id: i16,
    filename: DynamicPascalString,
}

impl FileReference {
    pub fn ty(&self) -> &str {
        self.ty.try_as_str().unwrap()
    }
    pub fn icon_id(&self) -> i16 {
        self.icon_id
    }
    pub fn filename(&self) -> &str {
        self.filename.as_str()
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ItemList {
    count_minus_one: u16,
    #[br(count = count_minus_one + 1)]
    list: Vec<Item>,
}

impl ItemList {
    pub fn items(&self) -> &[Item] {
        &self.list
    }
    pub fn items_mut(&mut self) -> &mut Vec<Item> {
        &mut self.list
    }
}

#[binrw::binread]
#[derive(Clone, Derivative, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Item {
    #[brw(pad_before = 4)]
    rect: Rect,
    ty: u8,
    #[br(args(ty))]
    item_type: ItemType,
}

impl Item {
    pub fn rect(&self) -> &Rect {
        &self.rect
    }
    pub fn enabled(&self) -> bool {
        self.ty & 0x80 != 0
    }
    pub fn data(&self) -> &ItemType {
        &self.item_type
    }
    pub fn data_mut(&mut self) -> &mut ItemType {
        &mut self.item_type
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
#[br(import(ty: u8))]
pub enum ItemType {
    #[br(pre_assert(ty & 0x7f == 4))]
    Button {
        #[brw(align_after = 2)]
        text: DynamicPascalString,
    },
    #[br(pre_assert(ty & 0x7f == 5))]
    Checkbox {
        #[brw(align_after = 2)]
        text: DynamicPascalString,
    },
    #[br(pre_assert(ty & 0x7f == 6))]
    RadioButton {
        #[brw(align_after = 2)]
        text: DynamicPascalString,
    },
    #[br(pre_assert(ty & 0x7f == 8))]
    StaticText {
        #[brw(align_after = 2)]
        text: DynamicPascalString,
    },
    #[br(pre_assert(ty & 0x7f == 16))]
    EditableText {
        #[brw(align_after = 2)]
        text: DynamicPascalString,
    },
    #[br(pre_assert(ty & 0x7f == 7))]
    Control {
        #[brw(pad_before = 1)]
        res: i16,
    },
    #[br(pre_assert(ty & 0x7f == 32))]
    Icon {
        #[brw(pad_before = 1)]
        res: i16,
    },
    #[br(pre_assert(ty & 0x7f == 64))]
    QuickDrawPicture {
        #[brw(pad_before = 1)]
        res: i16,
    },
    #[br(pre_assert(ty & 0x7f == 0))]
    AppDefined { pad: u8 },
    #[br(pre_assert(ty & 0x7f == 1))]
    Help {
        size: u8,
        helpitem: HelpItem,
    },
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum HelpItem {
    #[brw(magic = 1_u16)]
    ScanHdlg {
        hdlg_id: i16,
    },
    #[brw(magic = 2_u16)]
    ScanHrct {
        hrct_id: i16,
    },
    #[brw(magic = 8_u16)]
    ScanAppendHdlg {
        pad: i16,
        item_number: u16,
    },
}

use image::{GrayImage, ImageFormat, ImageResult};
use std::io;

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Cursor {
    img: Icon<32>,
    mask: Icon<32>,
    hotspot: Point,
}

impl Cursor {
    pub fn img_mut(&mut self) -> &mut Icon<32> {
        &mut self.img
    }
    pub fn mask_mut(&mut self) -> &mut Icon<32> {
        &mut self.mask
    }
    pub fn img(&self) -> &Icon<32> {
        &self.img
    }
    pub fn mask(&self) -> &Icon<32> {
        &self.mask
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ColorIcon<const SIZE: usize, const BPP: usize> {
    #[derivative(Debug = "ignore")]
    data: [u8; SIZE],
}

impl<const SIZE: usize, const BPP: usize> ColorIcon<SIZE, BPP> {
    pub const fn side(&self) -> usize {
        let pixels_per_bit: usize = 8 / BPP;
        (SIZE * pixels_per_bit).isqrt()
    }
}

impl<const SIZE: usize> ColorIcon<SIZE, 4> {
    const fn lut(v: u8) -> image::Rgb<u8> {
        lut::lut4(v)
    }
    pub fn write_to<W: io::Write + io::Seek>(&self, writer: &mut W) -> ImageResult<()> {
        self.image().write_to(writer, ImageFormat::Bmp)
    }
    pub fn image(&self) -> image::RgbImage {
        let data: Vec<u8> = self
            .data
            .view_bits::<Msb0>()
            .chunks(4)
            .flat_map(|v| Self::lut(v.load_be()).0)
            .collect();
        image::RgbImage::from_vec(self.side() as u32, self.side() as u32, data).unwrap()
    }
}

impl<const SIZE: usize> ColorIcon<SIZE, 8> {
    fn lut(v: u8, custom: &[(u16, image::Rgb<u16>)]) -> image::Rgb<u8> {
        custom
            .iter()
            .find_map(|(px, rgb)| {
                if *px == v as u16 {
                    let r = (rgb.0[0] >> 8) as u8;
                    let g = (rgb.0[1] >> 8) as u8;
                    let b = (rgb.0[2] >> 8) as u8;
                    Some(image::Rgb([r, g, b]))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| lut::lut8(v))
    }
    pub fn write_to<W: io::Write + io::Seek>(
        &self,
        writer: &mut W,
        lut: Option<&[(u16, image::Rgb<u16>)]>,
    ) -> ImageResult<()> {
        self.image(lut).write_to(writer, ImageFormat::Bmp)
    }
    pub fn image(&self, lut: Option<&[(u16, image::Rgb<u16>)]>) -> image::RgbImage {
        let data: Vec<u8> = self
            .data
            .view_bits::<Msb0>()
            .chunks(8)
            .flat_map(|v| Self::lut(v.load_be(), lut.unwrap_or(&[])).0)
            .collect();
        image::RgbImage::from_vec(self.side() as u32, self.side() as u32, data).unwrap()
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Icon<const SIZE: usize> {
    #[derivative(Debug = "ignore")]
    //data: BitArray<[u8; SIZE], Msb0>,
    data: [u8; SIZE],
}

impl<const SIZE: usize> Icon<SIZE> {
    pub const fn side(&self) -> usize {
        (SIZE * 8).isqrt()
    }
    pub fn write_to<W: io::Write + io::Seek>(&self, writer: &mut W) -> ImageResult<()> {
        self.image().write_to(writer, ImageFormat::Bmp)
    }
    //pub fn raw(&self) -> &[u8; SIZE] {
    //    &self.data
    //}
    pub fn set_pixel(&mut self, x: usize, y: usize, desired_value: bool) {
        let offset = x + y * self.side();
        let bit = offset % 8;
        let byte = offset / 8;
        if desired_value {
            self.data[byte] = self.data[byte] | 0x80 >> bit;
        } else {
            self.data[byte] = self.data[byte] & !(0x80 >> bit);
        }
    }
    pub fn pixel(&self, x: usize, y: usize) -> bool {
        let offset = x + y * self.side();
        self.data.view_bits::<Msb0>()[offset]
    }
    pub fn image(&self) -> image::GrayImage {
        let data: Vec<u8> = self
            .data
            .view_bits::<Msb0>()
            .iter()
            .by_vals()
            .map(|v| if v { 0xff } else { 0x00 })
            .collect();
        /*let data: Vec<u8> = self.data
        .iter()
        .flat_map(|b| [ b & 0x80, b & 0x40, b & 0x20, b & 0x10, b & 0x08, b & 0x04, b & 0x02, b & 0x01 ])
        .map(|b| if b == 0 { 0x00 } else { 0xff })
        .collect();*/
        GrayImage::from_raw(self.side() as u32, self.side() as u32, data).unwrap()
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct IconList<const SIZE: usize> {
    bw: Icon<SIZE>,
    mask: Icon<SIZE>,
}

impl<const SIZE: usize> IconList<SIZE> {
    pub const fn side(&self) -> usize {
        (SIZE * 8).isqrt()
    }
    pub fn bw(&self) -> &Icon<SIZE> {
        &self.bw
    }
    pub fn bw_mut(&mut self) -> &mut Icon<SIZE> {
        &mut self.bw
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Dialog {
    rect: Rect,
    window_def_id: i16,
    #[brw(pad_after = 1)]
    visibility: u8,
    #[brw(pad_after = 1)]
    close_box_spec: u8,
    reference_constant: u32,
    item_list_id: i16,
    #[brw(align_after = 2)]
    title: DynamicPascalString,
    #[br(try)]
    position: Option<Point>,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Alert {
    rect: Rect,
    item_list_res_id: u16,
    alert_info: u16,
    #[br(try)]
    position: Option<Point>,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Window {
    rect: Rect,
    id: u16,
    visibility: u16,
    close_box: u16,
    reference: u32,
    title: DynamicPascalString,
    //positioning: u16,
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Menu {
    #[brw(pad_after = 4)]
    id: u16,
    #[brw(pad_after = 2)]
    definition_res_id: u16,
    menu_state: u32,
    title: DynamicPascalString,
    #[br(parse_with = MenuItem::parse)]
    #[brw(pad_after = 1)]
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn id(&self) -> u16 {
        self.id
    }
    pub fn title_mut(&mut self) -> &mut String {
        self.title.as_mut()
    }
    pub fn items_mut(&mut self) -> &mut Vec<MenuItem> {
        &mut self.items
    }
    pub fn state_mut(&mut self) -> &mut u32 {
        &mut self.menu_state
    }
    pub fn state(&self) -> u32 {
        self.menu_state
    }
    pub fn set_state(&mut self, new: u32) {
        self.menu_state = new;
    }
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct MenuItem {
    text: DynamicPascalString,
    #[br(parse_with = MenuItemConfig::parser)]
    cfg: MenuItemConfig,
    style: Style,
}

impl MenuItem {
    pub fn new() -> MenuItem {
        MenuItem {
            text: DynamicPascalString::new(""),
            cfg: MenuItemConfig::Plain {
                icon: None,
                keyboard_shortcut: None,
                marking_character: None,
            },
            style: Style::new(),
        }
    }
    pub fn style(&self) -> Style {
        self.style
    }
    pub fn text_mut(&mut self) -> &mut String {
        self.text.as_mut()
    }
    pub fn cfg(&self) -> MenuItemConfig {
        self.cfg
    }
    #[binrw::parser(reader)]
    fn parse() -> BinResult<Vec<MenuItem>> {
        let mut vec = Vec::new();
        loop {
            let mut len: [u8; 1] = [0];
            reader.read_exact(&mut len)?;
            reader.seek(SeekFrom::Current(-1))?;
            if len[0] == 0 {
                break;
            }
            vec.push(MenuItem::read(reader)?);
        }
        Ok(vec)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MenuItemConfig {
    Plain {
        icon: Option<NonZeroU8>,
        keyboard_shortcut: Option<KeyboardShortcut>,
        marking_character: Option<MarkingCharacter>,
    },
    ScriptCode {
        code: NonZeroU8,
        marking_character: Option<MarkingCharacter>,
    },
    ReducedIcon {
        icon: NonZeroU8,
        marking_character: Option<MarkingCharacter>,
    },
    SicnIcon {
        icon: NonZeroU8,
        marking_character: Option<MarkingCharacter>,
    },
    Submenu {
        icon: Option<NonZeroU8>,
        submenu_id: u8,
    },
}

impl MenuItemConfig {
    #[binrw::parser(reader)]
    fn parser() -> BinResult<MenuItemConfig> {
        let mut bytes = [0, 0, 0];
        reader.read_exact(&mut bytes)?;
        let ret = if let Some(icon) = NonZeroU8::new(bytes[0]) {
            match bytes[1] {
                0x1b => MenuItemConfig::Submenu {
                    icon: Some(icon),
                    submenu_id: bytes[2],
                },
                0x1c => MenuItemConfig::ScriptCode {
                    code: icon,
                    marking_character: MarkingCharacter::new(bytes[2]),
                },
                0x1d => MenuItemConfig::ReducedIcon {
                    icon,
                    marking_character: MarkingCharacter::new(bytes[2]),
                },
                0x1e => MenuItemConfig::SicnIcon {
                    icon,
                    marking_character: MarkingCharacter::new(bytes[2]),
                },
                _ => MenuItemConfig::Plain {
                    icon: Some(icon),
                    keyboard_shortcut: KeyboardShortcut::from_repr(bytes[1]),
                    marking_character: MarkingCharacter::new(bytes[2]),
                },
            }
        } else {
            match bytes[1] {
                0x1b => MenuItemConfig::Submenu {
                    icon: None,
                    submenu_id: bytes[2],
                },
                _ => MenuItemConfig::Plain {
                    icon: None,
                    keyboard_shortcut: KeyboardShortcut::from_repr(bytes[1]),
                    marking_character: MarkingCharacter::new(bytes[2]),
                },
            }
        };
        Ok(ret)
    }
}

#[derive(Copy, Clone, Derivative, Eq, PartialEq, Hash)]
#[derivative(Debug)]
pub enum MarkingCharacter {
    Checkmark,
    FullDiamond,
    EmptyDiamond,
    Other(MacRoman),
}

impl MarkingCharacter {
    pub fn new(v: u8) -> Option<Self> {
        match v {
            0x00 => None,
            0x12 => Some(Self::Checkmark),
            _ => Some(Self::Other(MacRoman::from(v))),
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Checkmark => 0x12,
            Self::Other(v) => v.to_u8(),
            _ => todo!(),
        }
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct RomOverride {
    version: u16,
    number: u16,
    #[br(count = number)]
    override_types_ids: Vec<(u32, u16)>,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Size {
    #[br(map = |v: u16| SizeFlags::from_bits_retain(v))]
    #[bw(map = |v: &SizeFlags| v.bits())]
    flags: SizeFlags,
    preferred: u32,
    minimum: u32,
}

impl Size {
    pub fn flags(&self) -> SizeFlags {
        self.flags
    }
    pub fn minimum(&self) -> u32 {
        self.minimum
    }
    pub fn preferred(&self) -> u32 {
        self.preferred
    }
    pub fn minimum_mut(&mut self) -> &mut u32 {
        &mut self.minimum
    }
    pub fn preferred_mut(&mut self) -> &mut u32 {
        &mut self.preferred
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct SizeFlags: u16 {
        const SAVE_SCREEN = 0x8000;
        const ACCEPT_SUSPEND_EVENTS = 0x4000;
        const DISABLE_OPTION = 0x2000;
        const CAN_BACKGROUND = 0x1000;
        const DOES_ACTIVATE_ON_FG_SWITCH = 0x0800;
        const ONLY_BACKGROUND = 0x0400;
        const GET_FRONT_CLICKS = 0x0200;
        const ACCEPT_APP_DIED_EVENTS = 0x0100;
        const IS_32BIT_COMPATIBLE = 0x0080;
        const HIGH_LEVEL_EVENT_AWARE = 0x0040;
        const LOCAL_AND_REMOTE_HIGH_LEVEL_EVENTS = 0x0020;
        const STATIONERY_AWARE = 0x0010;
        const USE_TEXTEDIT_SERVICES = 0x0008;
    }
}

// Shamelessly stolen from
// https://github.com/zydeco/libmfs/blob/master/fobj.h
#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct MfsFolder {
    ty: MfsFolderType,
    icon_pos: Point,
    unk1: u32,
    view: u8,
    unk2: u8,
    parent: i16,
    unk3: [u8; 10],
    crtime: DateTime,
    mtime: DateTime,
    unk4: u16,
    bounds: Rect,
    scroll_offset: Point,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum MfsFolderType {
    #[brw(magic = b"\x00\x04")] Disk,
    #[brw(magic = b"\x00\x08")] Folder,
}

#[derive(Display, EnumIter, FromRepr, Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum KeyboardShortcut {
    A = 0x41,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}
