use crate::common::{DynamicPascalString, SizedString};
use binrw::{
    BinRead, BinResult, BinWrite,
    io::{Read, Seek, SeekFrom},
};
use bitflags::bitflags;
use derivative::Derivative;

pub mod types;

#[derive(Clone, Debug)]
pub struct Resource {
    id: i16,
    ty: ResourceType,
    pub system_heap: bool,
    pub purgeable: bool,
    pub locked: bool,
    pub protected: bool,
    pub preload: bool,
    pub compressed: bool,
    data: types::Type,
    name: Option<DynamicPascalString>,
}

impl Resource {
    pub fn id(&self) -> i16 {
        self.id
    }
    pub fn data_mut(&mut self) -> &mut types::Type {
        &mut self.data
    }
    pub fn data(&self) -> &types::Type {
        &self.data
    }
    pub fn ty(&self) -> &ResourceType {
        &self.ty
    }
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|v| v.try_as_str().unwrap())
    }
    pub fn read<R: Read + Seek>(reader: &mut R) -> BinResult<Vec<Resource>> {
        let raw = RawResource::read(reader)?;
        let mut ret = Vec::new();
        let mut ref_iter = raw.refs.iter();
        for t in raw.types.iter() {
            for r in raw.refs_of(t) {
                if r.attrs.contains(Attributes::COMPRESSED) {
                    println!("compressed, skipping...");
                }

                let data = types::Type::new(&t.ty, r.res_id, raw.data_of(r).to_owned())?;
                ret.push(Resource {
                    id: r.res_id,
                    ty: t.ty.clone(),
                    system_heap: r.attrs.contains(Attributes::SYSTEM_HEAP),
                    purgeable: r.attrs.contains(Attributes::PURGEABLE),
                    locked: r.attrs.contains(Attributes::LOCKED),
                    protected: r.attrs.contains(Attributes::PROTECTED),
                    preload: r.attrs.contains(Attributes::PRELOAD),
                    compressed: r.attrs.contains(Attributes::COMPRESSED),
                    data,
                    name: raw.name_of(r).cloned(),
                });
            }
        }

        Ok(ret)
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct RawResource {
    data_offset: u32,
    map_offset: u32,
    data_len: u32,
    map_len: u32,
    #[derivative(Debug = "ignore")]
    #[br(count = data_offset - 16)]
    system_data: Vec<u8>,
    #[derivative(Debug = "ignore")]
    #[br(count = data_len)]
    data: Vec<u8>,
    map: MapHeader,
    #[br(count = map.type_count_minus_one + 1)]
    types: Vec<Type>,
    #[br(count = types.iter().map(|t| t.ref_count_minus_one+1).sum::<u16>())]
    refs: Vec<Reference>,
    #[br(count = refs.iter().filter(|r| r.name_offset.is_some()).count())]
    names: Vec<DynamicPascalString>,
}

impl RawResource {
    fn refs_of(&self, ty: &Type) -> &[Reference] {
        let off = (ty.ref_list_offset as usize - self.types.len() * 8) / 12;
        let count = ty.ref_count_minus_one + 1;
        &self.refs[off..][..count as usize]
    }
    fn data_of(&self, r: &Reference) -> &[u8] {
        let off = r.data_offset as usize;
        let count = u32::from_be_bytes([
            self.data[off],
            self.data[off + 1],
            self.data[off + 2],
            self.data[off + 3],
        ]) as usize;
        &self.data[off + 4..][..count]
    }
    fn name_of(&self, r: &Reference) -> Option<&DynamicPascalString> {
        let mut off = r.name_offset? as usize;
        for name in self.names.iter() {
            if off == 0 {
                return Some(name);
            }
            off -= 1;
            off -= name.len();
        }

        None
    }
}

#[derive(Clone, Derivative, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct MapHeader {
    #[derivative(Debug = "ignore")]
    _reserved_hdr_copy: [u8; 16],
    #[derivative(Debug = "ignore")]
    _reserved_handle_next_map: u32,
    #[derivative(Debug = "ignore")]
    _reserved_file_reference: u16,
    fork_attrs: u16,
    type_list_offset: u16,
    name_list_offset: u16,
    type_count_minus_one: u16,
}

#[derive(Clone, Derivative, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct Type {
    #[br(map = |string: SizedString<4>| ResourceType::from(string))]
    #[bw(map = |ty: &ResourceType| SizedString::<4>::from(ty.clone()))]
    ty: ResourceType,
    ref_count_minus_one: u16,
    ref_list_offset: u16,
}

#[derive(Clone, Derivative, BinRead, BinWrite)]
#[derivative(Debug)]
#[brw(big)]
pub struct Reference {
    res_id: i16,
    #[br(map = |v: u16| (v != 0xffff).then_some(v))]
    #[bw(map = |v: &Option<u16>| v.unwrap_or(0xffff) )]
    name_offset: Option<u16>,
    #[br(map = |v: u8| Attributes::from_bits_retain(v))]
    #[bw(map = |v: &Attributes| v.bits())]
    attrs: Attributes,
    #[br(parse_with = binrw::helpers::read_u24)]
    #[bw(write_with = binrw::helpers::write_u24)]
    data_offset: u32,
    #[derivative(Debug = "ignore")]
    _reserved_handle: u32,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct Attributes: u8 {
        const COMPRESSED = 0x1;
        const WRITE_TO_RESOURCE_FILE = 0x2;
        const PRELOAD = 0x4;
        const PROTECTED = 0x8;
        const LOCKED = 0x10;
        const PURGEABLE = 0x20;
        const SYSTEM_HEAP = 0x40;
        const SYSTEM_REFERENCE = 0x80;
    }
}

macro_rules! sized_string_enum {
    (
        $name: ident, $size: literal,
        $(($code: literal => $variant: ident),)*
    ) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum $name {
            Other(SizedString<$size>),
            $($variant),*
        }
        impl From<SizedString<$size>> for $name {
            fn from(f: SizedString<$size>) -> $name {
                match f.as_inner() {
                    $($code => <$name>::$variant,)*
                    _ => <$name>::Other(f),
                }
            }
        }
        impl From<$name> for SizedString<$size> {
            fn from(f: $name) -> SizedString<$size> {
                match f {
                    $(<$name>::$variant => (*$code).into(),)*
                    $name::Other(other) => other,
                }
            }
        }
    }
}

sized_string_enum!(
    ResourceType, 4,
    (b"ALRT" => AlertBoxTemplate),
    (b"BNDL" => Bundle),
    (b"CODE" => Code),
    (b"CURS" => Cursor),
    (b"crsr" => ColorCursor),
    (b"DITL" => ItemList),
    (b"DLOG" => DialogBoxTemplate),
    (b"FONT" => BitmapFont),
    (b"ICON" => Icon32),
    (b"icon" => Icon),
    (b"ICN#" => FinderIcon),
    (b"FREF" => FileReference),
    (b"SICN" => SmallIcons),
    (b"cicn" => ColorIcons),
    (b"MBAR" => MenuBar),
    (b"MENU" => Menu),
    (b"PAT " => Pattern),
    (b"PAT#" => PatternList),
    (b"PICT" => QuickDrawPicture),
    (b"SIZE" => Size),
    (b"STR " => String),
    (b"STR#" => StringList),
    (b"MACS" => SystemVersion),
    (b"WIND" => WindowTemplate),
    (b"hdlg" => DialogOrAlertBoxHelp),
    (b"sfnt" => OutlineFont),
    (b"snd " => Sound),
    (b"CACH" => RamCache),
    (b"DSAT" => StartupAlertTable),
    (b"FCMT" => GetInfoComments),
    (b"FMTR" => FloppyFormattingCode),
    (b"FOBJ" => MfsFolderInfo),
    (b"FRSV" => SystemFontIds),
    (b"KMAP" => HwKeyboardMap),
    (b"MBDF" => DefaultMenuDefinition),
    (b"MMAP" => MouseTrackingCode),
    (b"NBPC" => AppleTalkBundle),
    (b"PDEF" => PrintingCode),
    (b"PTCH" => RomPatch),
    (b"ROv#" => RomResourceOverrideList),
    (b"ROvr" => RomResourceOverrideCode),
    (b"ictb" => ItemColorTable),
    (b"itl0" => DateTimeFormats),
    (b"itl1" => DayMonthNames),
    (b"itl2" => TextUtilSortHooks),
    (b"itl4" => LocalizableTablesAndCode),
    (b"itlk" => EarlyKeyRemap),
    (b"kcs#" => SmallBwIconList),
    (b"kcs4" => Small4BitColorIcon),
    (b"kcd8" => Small8BitColorIcon),
    (b"mctb" => MenuColorInfoTable),
    (b"mntr" => MonitorsExtensionCode),
    (b"movv" => QuickTimeMovie),
    (b"pltt" => ColorPalette),
    (b"ppat" => PixelPattern),
    (b"qdef" => QueryDefinitionFn),
    (b"qrsc" => Query),
    (b"sect" => SectionRecord),
    (b"snth" => Synthesizer),
    (b"styl" => TextEditStyle),
    (b"sysz" => SystemHeapSpaceRequired),
    (b"vers" => VersionNumber),
    (b"wctb" => WindowColorTable),
    (b"wstr" => LongString),
    (b"PACK" => ToolboxPackage),
    (b"FKEY" => FunctionKey),
    (b"DRVR" => Driver),
    (b"MDEF" => MenuDefinitionProcedure),
    (b"CDEF" => ControlDefinitionFn),
    (b"INIT" => SystemExtension),
    (b"WDEF" => WindowDefinition),
    (b"INTL" => InternationalObsolete),
    (b"PREC" => PrintRecord),
    (b"ics#" => SmallIconList),
    (b"icl4" => LargeColorIcon4),
    (b"ics4" => SmallColorIcon4),
    (b"icl8" => LargeColorIcon8),
    (b"ics8" => SmallColorIcon8),
    (b"FOND" => FontFamilyRecord),
    (b"NFNT" => Rom128kFont),
    (b"PRER" => ChooserNonSerialPrinter),
    (b"PRES" => ChooserSerialPrinter),
    (b"RDEV" => ChooserOtherDevice),
    (b"bmap" => ControlPanelBitmap),
    (b"ctab" => ControlPanelThing),
    (b"insc" => InstallerScript),
    (b"LDEF" => ListDefinitionProcedure),
    (b"ADBS" => AdbServiceRoutine),
    (b"KCAP" => KeyboardPhysicalLayout),
    (b"KCHR" => KeyboardMappingSoftware),
    (b"KSWP" => KeyboardScriptTable),
    (b"actb" => AlertColorTable),
    (b"atpl" => AppleTalkInternal),
    (b"boot" => BootBlocks),
    (b"cctb" => ControlColorTable),
    (b"clst" => CachedIconLists),
    (b"clut" => ColorLut),
    (b"dctb" => DialogColorTable),
    (b"fctb" => FontColorTable),
    (b"gama" => ColorCorrectionTable),
    (b"lmem" => LowMemoryGlobals),
    (b"mcky" => MouseTracking),
    (b"mitq" => MakeITableMemoryRequirements),
    (b"mppc" => AppleTalkConfig),
    (b"nrct" => RectanglePositions),
    (b"scrn" => Screen),
    (b"CNTL" => Control),
    (b"acur" => AnimatedCursor),
    (b"TMPL" => Template),
);
