use binrw::{BinRead, BinWrite, BinResult};
use binrw::io::{Read, Seek, SeekFrom};
use crate::common::{DateTime, DynamicPascalString, SizedString, Rect, Point};
use crate::i18n::RegionCode;
use derivative::Derivative;
use super::ResourceType;

#[derive(Clone, Derivative, Eq, PartialEq)]
#[derivative(Debug)]
pub enum Type {
    Menu(Menu),
    String(DynamicPascalString),
    RomOverride(RomOverride),
    MfsFolder(MfsFolder),
    Window(Window),
    Alert(Alert),
    Size(Size),
    FinderIcon(IconList<128>),
    SmallIcon(IconList<32>),
    Icon(Icon<128>),
    FileReference(FileReference),
    ItemList(ItemList),
    Version(Version),
    Other(
        #[derivative(Debug = "ignore")]
        Vec<u8>
    ),
}

impl Type {
    pub fn new(kind: &ResourceType, data: Vec<u8>) -> BinResult<Type> {
        let mut cursor = std::io::Cursor::new(data);
        Ok(match kind {
            ResourceType::Menu => Type::Menu(Menu::read(&mut cursor)?),
            ResourceType::RomResourceOverrideList => Type::RomOverride(RomOverride::read(&mut cursor)?),
            ResourceType::String => Type::String(DynamicPascalString::read(&mut cursor)?),
            ResourceType::MfsFolderInfo => Type::MfsFolder(MfsFolder::read(&mut cursor)?),
            ResourceType::Size => Type::Size(Size::read(&mut cursor)?),
            ResourceType::WindowTemplate => Type::Window(Window::read(&mut cursor)?),
            ResourceType::FinderIcon => Type::FinderIcon(IconList::read(&mut cursor)?),
            ResourceType::SmallIconList => Type::SmallIcon(IconList::read(&mut cursor)?),
            ResourceType::AlertBoxTemplate => Type::Alert(Alert::read(&mut cursor)?),
            ResourceType::Icon => Type::Icon(Icon::read(&mut cursor)?),
            ResourceType::ItemList => Type::ItemList(ItemList::read(&mut cursor)?),
            ResourceType::VersionNumber => Type::Version(Version::read(&mut cursor)?),
            ResourceType::FileReference => Type::FileReference(FileReference::read(&mut cursor)?),
            _ => Type::Other(cursor.into_inner()),
        })
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Version {
    minor: u8,
    major: u8,
    development_stage: DevelopmentStage,
    prerelease_revision: u8,
    #[brw(pad_before = 1)]
    region: RegionCode,
    version_number: DynamicPascalString,
    version_message: DynamicPascalString,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub enum DevelopmentStage {
    #[brw(magic = 0x20_u8)] PreAlpha,
    #[brw(magic = 0x40_u8)] Alpha,
    #[brw(magic = 0x60_u8)] Beta,
    #[brw(magic = 0x80_u8)] Released,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct FileReference {
    ty: SizedString<4>,
    #[brw(pad_after = 1)]
    local_id: u32,
}

#[derive(Clone, Derivative, BinRead, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct ItemList {
    count_minus_one: u16,
    #[br(count = count_minus_one + 1)]
    list: Vec<Item>,
}

#[binrw::binread]
#[derive(Clone, Derivative, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Item {
    #[brw(pad_before = 4)]
    rect: Rect,
    #[br(temp)]
    #[brw(dbg)]
    ty: u8,
    #[br(args(ty))]
    item_type: ItemType,
    #[br(calc = ty & 0x80 != 0)]
    enabled: bool,
    #[brw(align_after = 2)]
    text: DynamicPascalString,
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
    AppDefined {
        pad: u8,
    },
    // TODO
    #[br(pre_assert(ty & 0x7f == 1))]
    Help {
    },
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Icon<const SIZE: usize> {
    #[derivative(Debug = "ignore")]
    data: [u8; SIZE],
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct IconList<const SIZE: usize> {
    #[derivative(Debug = "ignore")]
    bw: [u8; SIZE],
    #[derivative(Debug = "ignore")]
    mask: [u8; SIZE],
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct Alert {
    rect: Rect,
    item_list_res_id: u16,
    alert_info: u16,
    // position???
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
    positioning: u16,
}

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
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

#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct MenuItem {
    text: DynamicPascalString,
    icon_or_script: u8,
    kbd_eq: u8,
    marking_char_or_id: u8,
    style: u8,
}

impl MenuItem {
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
    flags: u16,
    minimum: u32,
    preferred: u32,
}

// Shamelessly stolen from
// https://github.com/zydeco/libmfs/blob/master/fobj.h
#[derive(Clone, Derivative, BinRead, BinWrite, Eq, PartialEq)]
#[derivative(Debug)]
#[brw(big)]
pub struct MfsFolder {
    ty: u16,
    icon_pos: Point,
    unk1: u32,
    unk2: u16,
    parent: u16,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    crtime: DateTime,
    mtime: DateTime,
    backup_date: DateTime,
    flags: u16,
    unk6: u8,
}
