//! Minimal FFI to AMD ADLX (`amdadlx64.dll`) for manual GPU tuning.
//!
//! Only the interfaces/methods needed to read and set GPU core-clock tuning are
//! bound. Vtable layouts are transcribed IN DECLARATION ORDER from the official
//! SDK headers (GPUOpen-LibrariesAndSDKs/ADLX, SDK/Include/*.h); unused slots
//! are kept as `*const c_void` placeholders so the byte offsets of the methods
//! we DO call stay correct. Getting a slot order wrong = calling the wrong
//! function pointer, so do not reorder these structs.
//!
//! Target: Navi (RDNA2+) uses IADLXManualGraphicsTuning2 (min/max freq +
//! voltage). On Navi4+ (RX 9070 XT) SetGPUMaxFrequency is an OFFSET, not an
//! absolute clock -- min frequency (what we set here) is still absolute.

#![allow(non_snake_case, non_camel_case_types, dead_code, unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_void, CStr};

// ADLX scalar typedefs. NOTE: adlx_long == C `long` == 32-bit on Windows LLP64.
type ADLX_RESULT = i32;
type adlx_long = i32;
type adlx_int = i32;
type adlx_uint = u32;
type adlx_bool = u8;

const ADLX_OK: ADLX_RESULT = 0;

pub fn result_name(r: ADLX_RESULT) -> &'static str {
    match r {
        0 => "ADLX_OK",
        1 => "ADLX_ALREADY_ENABLED",
        2 => "ADLX_ALREADY_INITIALIZED",
        3 => "ADLX_FAIL",
        4 => "ADLX_INVALID_ARGS",
        5 => "ADLX_BAD_VER",
        6 => "ADLX_UNKNOWN_INTERFACE",
        7 => "ADLX_TERMINATED",
        8 => "ADLX_ADL_INIT_ERROR",
        9 => "ADLX_NOT_FOUND",
        10 => "ADLX_INVALID_OBJECT",
        11 => "ADLX_ORPHAN_OBJECTS",
        12 => "ADLX_NOT_SUPPORTED",
        13 => "ADLX_PENDING_OPERATION",
        14 => "ADLX_GPU_INACTIVE",
        15 => "ADLX_GPU_IN_USE",
        16 => "ADLX_TIMEOUT_OPERATION",
        17 => "ADLX_NOT_ACTIVE",
        18 => "ADLX_RESET_NEEDED",
        _ => "ADLX_<unknown>",
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AdlxIntRange {
    pub min: adlx_int,
    pub max: adlx_int,
    pub step: adlx_int,
}

// ---- IADLXSystem (ISystem.h) -- no Acquire/Release, must NOT be released ----
#[repr(C)]
struct IADLXSystem {
    vtbl: *const IADLXSystemVtbl,
}
#[repr(C)]
struct IADLXSystemVtbl {
    GetHybridGraphicsType: *const c_void, // 1
    GetGPUs: unsafe extern "C" fn(*mut IADLXSystem, *mut *mut IADLXGPUList) -> ADLX_RESULT, // 2
    QueryInterface: *const c_void,        // 3
    GetDisplaysServices: *const c_void,   // 4
    GetDesktopsServices: *const c_void,   // 5
    GetGPUsChangedHandling: *const c_void, // 6
    EnableLog: *const c_void,             // 7
    Get3DSettingsServices: *const c_void, // 8
    GetGPUTuningServices:
        unsafe extern "C" fn(*mut IADLXSystem, *mut *mut IADLXGPUTuningServices) -> ADLX_RESULT, // 9
    GetPerformanceMonitoringServices: *const c_void, // 10
    TotalSystemRAM: *const c_void,        // 11
    GetI2C: *const c_void,                // 12
}

// ---- IADLXGPUList (ISystem.h) ----
#[repr(C)]
struct IADLXGPUList {
    vtbl: *const IADLXGPUListVtbl,
}
#[repr(C)]
struct IADLXGPUListVtbl {
    Acquire: *const c_void,                                            // 1
    Release: unsafe extern "C" fn(*mut IADLXGPUList) -> adlx_long,     // 2
    QueryInterface: *const c_void,                                     // 3
    Size: *const c_void,                                              // 4
    Empty: *const c_void,                                            // 5
    Begin: unsafe extern "C" fn(*mut IADLXGPUList) -> adlx_uint,      // 6
    End: *const c_void,                                              // 7
    At: *const c_void,                                              // 8
    Clear: *const c_void,                                            // 9
    Remove_Back: *const c_void,                                      // 10
    Add_Back: *const c_void,                                         // 11
    At_GPUList:
        unsafe extern "C" fn(*mut IADLXGPUList, adlx_uint, *mut *mut IADLXGPU) -> ADLX_RESULT, // 12
    Add_Back_GPUList: *const c_void,                                 // 13
}

// ---- IADLXGPU (ISystem.h) ----
#[repr(C)]
struct IADLXGPU {
    vtbl: *const IADLXGPUVtbl,
}
#[repr(C)]
struct IADLXGPUVtbl {
    Acquire: *const c_void,                                       // 1
    Release: unsafe extern "C" fn(*mut IADLXGPU) -> adlx_long,    // 2
    QueryInterface: *const c_void,                                // 3
    VendorId: *const c_void,                                      // 4
    ASICFamilyType: *const c_void,                                // 5
    Type: *const c_void,                                          // 6
    IsExternal: *const c_void,                                    // 7
    Name: unsafe extern "C" fn(*mut IADLXGPU, *mut *const c_char) -> ADLX_RESULT, // 8
    DriverPath: *const c_void,                                    // 9
    PNPString: *const c_void,                                     // 10
    HasDesktops: *const c_void,                                   // 11
    TotalVRAM: *const c_void,                                     // 12
    VRAMType: *const c_void,                                      // 13
    BIOSInfo: *const c_void,                                      // 14
    DeviceId: unsafe extern "C" fn(*mut IADLXGPU, *mut *const c_char) -> ADLX_RESULT, // 15
    RevisionId: *const c_void,                                    // 16
    SubSystemId: *const c_void,                                   // 17
    SubSystemVendorId: *const c_void,                             // 18
    UniqueId: *const c_void,                                      // 19
}

// ---- IADLXGPUTuningServices (IGPUTuning.h) ----
#[repr(C)]
struct IADLXGPUTuningServices {
    vtbl: *const IADLXGPUTuningServicesVtbl,
}
#[repr(C)]
struct IADLXGPUTuningServicesVtbl {
    Acquire: *const c_void,                                                  // 1
    Release: unsafe extern "C" fn(*mut IADLXGPUTuningServices) -> adlx_long, // 2
    QueryInterface: *const c_void,                                           // 3
    GetGPUTuningChangedHandling: *const c_void,                             // 4
    IsAtFactory:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut adlx_bool) -> ADLX_RESULT, // 5
    ResetToFactory:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU) -> ADLX_RESULT, // 6
    IsSupportedAutoTuning: *const c_void,                                   // 7
    IsSupportedPresetTuning: *const c_void,                                 // 8
    IsSupportedManualGFXTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut adlx_bool) -> ADLX_RESULT, // 9
    IsSupportedManualVRAMTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut adlx_bool) -> ADLX_RESULT, // 10
    IsSupportedManualFanTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut adlx_bool) -> ADLX_RESULT, // 11
    IsSupportedManualPowerTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut adlx_bool) -> ADLX_RESULT, // 12
    GetAutoTuning: *const c_void,                                           // 13
    GetPresetTuning: *const c_void,                                         // 14
    GetManualGFXTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut *mut IADLXInterface) -> ADLX_RESULT, // 15
    GetManualVRAMTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut *mut IADLXInterface) -> ADLX_RESULT, // 16
    GetManualFanTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut *mut IADLXInterface) -> ADLX_RESULT, // 17
    GetManualPowerTuning:
        unsafe extern "C" fn(*mut IADLXGPUTuningServices, *mut IADLXGPU, *mut *mut IADLXInterface) -> ADLX_RESULT, // 18
}

// ---- IADLXInterface (generic base; ADLXDefines.h) ----
#[repr(C)]
struct IADLXInterface {
    vtbl: *const IADLXInterfaceVtbl,
}
#[repr(C)]
struct IADLXInterfaceVtbl {
    Acquire: *const c_void,                                        // 1
    Release: unsafe extern "C" fn(*mut IADLXInterface) -> adlx_long, // 2
    QueryInterface:
        unsafe extern "C" fn(*mut IADLXInterface, *const u16, *mut *mut c_void) -> ADLX_RESULT, // 3
}

// ---- IADLXManualGraphicsTuning2 (IGPUManualGFXTuning.h) ----
#[repr(C)]
struct IADLXManualGraphicsTuning2 {
    vtbl: *const IADLXManualGraphicsTuning2Vtbl,
}
#[repr(C)]
struct IADLXManualGraphicsTuning2Vtbl {
    Acquire: *const c_void,                                                       // 1
    Release: unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2) -> adlx_long,  // 2
    QueryInterface: *const c_void,                                                // 3
    GetGPUMinFrequencyRange:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, *mut AdlxIntRange) -> ADLX_RESULT, // 4
    GetGPUMinFrequency:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, *mut adlx_int) -> ADLX_RESULT, // 5
    SetGPUMinFrequency:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, adlx_int) -> ADLX_RESULT, // 6
    GetGPUMaxFrequencyRange:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, *mut AdlxIntRange) -> ADLX_RESULT, // 7
    GetGPUMaxFrequency:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, *mut adlx_int) -> ADLX_RESULT, // 8
    SetGPUMaxFrequency:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, adlx_int) -> ADLX_RESULT, // 9
    GetGPUVoltageRange:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, *mut AdlxIntRange) -> ADLX_RESULT, // 10
    GetGPUVoltage:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, *mut adlx_int) -> ADLX_RESULT, // 11
    SetGPUVoltage:
        unsafe extern "C" fn(*mut IADLXManualGraphicsTuning2, adlx_int) -> ADLX_RESULT, // 12
}

// ---- IADLXManualPowerTuning (IGPUManualPowerTuning.h) ----
#[repr(C)]
struct IADLXManualPowerTuning {
    vtbl: *const IADLXManualPowerTuningVtbl,
}
#[repr(C)]
struct IADLXManualPowerTuningVtbl {
    Acquire: *const c_void,                                                    // 1
    Release: unsafe extern "C" fn(*mut IADLXManualPowerTuning) -> adlx_long,   // 2
    QueryInterface: *const c_void,                                             // 3
    GetPowerLimitRange:
        unsafe extern "C" fn(*mut IADLXManualPowerTuning, *mut AdlxIntRange) -> ADLX_RESULT, // 4
    GetPowerLimit:
        unsafe extern "C" fn(*mut IADLXManualPowerTuning, *mut adlx_int) -> ADLX_RESULT, // 5
    SetPowerLimit:
        unsafe extern "C" fn(*mut IADLXManualPowerTuning, adlx_int) -> ADLX_RESULT, // 6
    IsSupportedTDCLimit: *const c_void,                                        // 7
    GetTDCLimitRange: *const c_void,                                           // 8
    GetTDCLimit: *const c_void,                                                // 9
    SetTDCLimit: *const c_void,                                                // 10
}

// ---- IADLXManualVRAMTuning2 (IGPUManualVRAMTuning.h) -- Navi memory clock ----
#[repr(C)]
struct IADLXManualVRAMTuning2 {
    vtbl: *const IADLXManualVRAMTuning2Vtbl,
}
#[repr(C)]
struct IADLXManualVRAMTuning2Vtbl {
    Acquire: *const c_void,                                                     // 1
    Release: unsafe extern "C" fn(*mut IADLXManualVRAMTuning2) -> adlx_long,    // 2
    QueryInterface: *const c_void,                                             // 3
    IsSupportedMemoryTiming: *const c_void,                                    // 4
    GetSupportedMemoryTimingDescriptionList: *const c_void,                    // 5
    GetMemoryTimingDescription: *const c_void,                                 // 6
    SetMemoryTimingDescription: *const c_void,                                 // 7
    GetMaxVRAMFrequencyRange:
        unsafe extern "C" fn(*mut IADLXManualVRAMTuning2, *mut AdlxIntRange) -> ADLX_RESULT, // 8
    GetMaxVRAMFrequency:
        unsafe extern "C" fn(*mut IADLXManualVRAMTuning2, *mut adlx_int) -> ADLX_RESULT, // 9
    SetMaxVRAMFrequency:
        unsafe extern "C" fn(*mut IADLXManualVRAMTuning2, adlx_int) -> ADLX_RESULT, // 10
}

// ---- IADLXManualFanTuningState (IGPUManualFanTuning.h) -- one curve point ----
#[repr(C)]
struct IADLXManualFanTuningState {
    vtbl: *const IADLXManualFanTuningStateVtbl,
}
#[repr(C)]
struct IADLXManualFanTuningStateVtbl {
    Acquire: *const c_void,                                                          // 1
    Release: unsafe extern "C" fn(*mut IADLXManualFanTuningState) -> adlx_long,      // 2
    QueryInterface: *const c_void,                                                   // 3
    GetFanSpeed: *const c_void,                                                      // 4
    SetFanSpeed:
        unsafe extern "C" fn(*mut IADLXManualFanTuningState, adlx_int) -> ADLX_RESULT, // 5
    GetTemperature: *const c_void,                                                   // 6
    SetTemperature:
        unsafe extern "C" fn(*mut IADLXManualFanTuningState, adlx_int) -> ADLX_RESULT, // 7
}

// ---- IADLXManualFanTuningStateList (IGPUManualFanTuning.h) ----
#[repr(C)]
struct IADLXManualFanTuningStateList {
    vtbl: *const IADLXManualFanTuningStateListVtbl,
}
#[repr(C)]
struct IADLXManualFanTuningStateListVtbl {
    Acquire: *const c_void,                                                        // 1
    Release: unsafe extern "C" fn(*mut IADLXManualFanTuningStateList) -> adlx_long, // 2
    QueryInterface: *const c_void,                                                 // 3
    Size: unsafe extern "C" fn(*mut IADLXManualFanTuningStateList) -> adlx_uint,   // 4
    Empty: *const c_void,                                                          // 5
    Begin: *const c_void,                                                          // 6
    End: *const c_void,                                                            // 7
    At: *const c_void,                                                             // 8
    Clear: *const c_void,                                                          // 9
    Remove_Back: *const c_void,                                                    // 10
    Add_Back: *const c_void,                                                       // 11
    At_ManualFanTuningStateList: unsafe extern "C" fn(
        *mut IADLXManualFanTuningStateList,
        adlx_uint,
        *mut *mut IADLXManualFanTuningState,
    ) -> ADLX_RESULT, // 12
    Add_Back_ManualFanTuningStateList: *const c_void,                              // 13
}

// ---- IADLXManualFanTuning (IGPUManualFanTuning.h) ----
#[repr(C)]
struct IADLXManualFanTuning {
    vtbl: *const IADLXManualFanTuningVtbl,
}
#[repr(C)]
struct IADLXManualFanTuningVtbl {
    Acquire: *const c_void,                                                     // 1
    Release: unsafe extern "C" fn(*mut IADLXManualFanTuning) -> adlx_long,      // 2
    QueryInterface: *const c_void,                                              // 3
    GetFanTuningRanges: unsafe extern "C" fn(
        *mut IADLXManualFanTuning,
        *mut AdlxIntRange, // speed
        *mut AdlxIntRange, // temperature
    ) -> ADLX_RESULT, // 4
    GetFanTuningStates: unsafe extern "C" fn(
        *mut IADLXManualFanTuning,
        *mut *mut IADLXManualFanTuningStateList,
    ) -> ADLX_RESULT, // 5
    GetEmptyFanTuningStates: *const c_void,                                     // 6
    IsValidFanTuningStates: unsafe extern "C" fn(
        *mut IADLXManualFanTuning,
        *mut IADLXManualFanTuningStateList,
        *mut adlx_int, // errorIndex
    ) -> ADLX_RESULT, // 7
    SetFanTuningStates: unsafe extern "C" fn(
        *mut IADLXManualFanTuning,
        *mut IADLXManualFanTuningStateList,
    ) -> ADLX_RESULT, // 8
    // remaining slots (ZeroRPM, MinAcoustic, MinFanSpeed, TargetFanSpeed) unused
    IsSupportedZeroRPM: *const c_void,                                          // 9
    GetZeroRPMState: *const c_void,                                             // 10
    SetZeroRPMState: *const c_void,                                             // 11
    IsSupportedMinAcousticLimit: *const c_void,                                 // 12
    GetMinAcousticLimitRange: *const c_void,                                    // 13
    GetMinAcousticLimit: *const c_void,                                         // 14
    SetMinAcousticLimit: *const c_void,                                         // 15
    IsSupportedMinFanSpeed: *const c_void,                                      // 16
    GetMinFanSpeedRange: *const c_void,                                         // 17
    GetMinFanSpeed: *const c_void,                                              // 18
    SetMinFanSpeed: *const c_void,                                              // 19
    IsSupportedTargetFanSpeed: *const c_void,                                   // 20
    GetTargetFanSpeedRange: *const c_void,                                      // 21
    GetTargetFanSpeed: *const c_void,                                           // 22
    SetTargetFanSpeed: *const c_void,                                           // 23
}

// Bootstrap export signatures (ADLX.h). __cdecl, but on x64 == extern "C".
type ADLXQueryFullVersion_Fn = unsafe extern "C" fn(*mut u64) -> ADLX_RESULT;
type ADLXInitialize_Fn = unsafe extern "C" fn(u64, *mut *mut IADLXSystem) -> ADLX_RESULT;
type ADLXTerminate_Fn = unsafe extern "C" fn() -> ADLX_RESULT;

/// Live view of the GPU's manual tuning (core clock, voltage, power).
#[derive(Debug, Clone)]
pub struct GfxTuning {
    pub gpu_name: String,
    pub device_id: String,
    pub min_range: AdlxIntRange,
    pub max_range: AdlxIntRange,
    pub volt_range: AdlxIntRange,
    pub cur_min: adlx_int,
    pub cur_max: adlx_int,
    pub cur_volt: adlx_int,
    /// Power limit is a separate ADLX interface; None if not supported.
    pub power_range: Option<AdlxIntRange>,
    pub cur_power: Option<adlx_int>,
    /// Memory/VRAM clock, separate interface; None if not supported.
    pub mem_range: Option<AdlxIntRange>,
    pub cur_mem: Option<adlx_int>,
}

/// A set of tuning values to apply. `None` fields are left untouched.
#[derive(Debug, Clone, Default)]
pub struct TuningSet {
    pub min_clock: Option<i32>,
    pub max_clock: Option<i32>,
    pub voltage: Option<i32>,
    pub mem_clock: Option<i32>,
    pub power_limit: Option<i32>,
    /// Fan curve as (temperature C, fan speed %) points, low->high. None = leave
    /// fan control untouched.
    pub fan_curve: Option<Vec<(i32, i32)>>,
}

/// RAII session: keeps the DLL loaded and terminates ADLX on drop.
pub struct Adlx {
    _lib: libloading::Library,
    terminate: ADLXTerminate_Fn,
    system: *mut IADLXSystem,
}

impl Adlx {
    pub fn init() -> Result<Self, String> {
        unsafe {
            let lib = libloading::Library::new("amdadlx64.dll")
                .map_err(|e| format!("loading amdadlx64.dll: {e}"))?;

            let query_ver: libloading::Symbol<ADLXQueryFullVersion_Fn> = lib
                .get(b"ADLXQueryFullVersion\0")
                .map_err(|e| format!("resolving ADLXQueryFullVersion: {e}"))?;
            let initialize: libloading::Symbol<ADLXInitialize_Fn> = lib
                .get(b"ADLXInitialize\0")
                .map_err(|e| format!("resolving ADLXInitialize: {e}"))?;
            let terminate: libloading::Symbol<ADLXTerminate_Fn> = lib
                .get(b"ADLXTerminate\0")
                .map_err(|e| format!("resolving ADLXTerminate: {e}"))?;

            // Use the DLL's own version so init never fails on ADLX_BAD_VER.
            let mut full_ver: u64 = 0;
            let r = query_ver(&mut full_ver);
            if r != ADLX_OK {
                return Err(format!("ADLXQueryFullVersion -> {}", result_name(r)));
            }

            let mut system: *mut IADLXSystem = std::ptr::null_mut();
            let r = initialize(full_ver, &mut system);
            if r != ADLX_OK || system.is_null() {
                return Err(format!("ADLXInitialize -> {}", result_name(r)));
            }

            let terminate: ADLXTerminate_Fn = *terminate;
            Ok(Adlx {
                _lib: lib,
                terminate,
                system,
            })
        }
    }

    /// Get the first GPU pointer. Caller must Release it.
    unsafe fn first_gpu(&self) -> Result<*mut IADLXGPU, String> {
        let sysv = &*(*self.system).vtbl;
        let mut list: *mut IADLXGPUList = std::ptr::null_mut();
        let r = (sysv.GetGPUs)(self.system, &mut list);
        if r != ADLX_OK || list.is_null() {
            return Err(format!("GetGPUs -> {}", result_name(r)));
        }
        let listv = &*(*list).vtbl;
        let begin = (listv.Begin)(list);
        let mut gpu: *mut IADLXGPU = std::ptr::null_mut();
        let r = (listv.At_GPUList)(list, begin, &mut gpu);
        (listv.Release)(list);
        if r != ADLX_OK || gpu.is_null() {
            return Err(format!("At_GPUList -> {}", result_name(r)));
        }
        Ok(gpu)
    }

    unsafe fn gpu_string(
        gpu: *mut IADLXGPU,
        f: unsafe extern "C" fn(*mut IADLXGPU, *mut *const c_char) -> ADLX_RESULT,
    ) -> String {
        let mut p: *const c_char = std::ptr::null();
        if f(gpu, &mut p) == ADLX_OK && !p.is_null() {
            CStr::from_ptr(p).to_string_lossy().into_owned()
        } else {
            String::new()
        }
    }

    /// Acquire the manual GFX tuning interface for a GPU (Navi/Tuning2).
    /// Returns the tuning2 pointer and the tuning-services pointer; caller
    /// releases both.
    unsafe fn manual_gfx2(
        &self,
        gpu: *mut IADLXGPU,
    ) -> Result<(*mut IADLXManualGraphicsTuning2, *mut IADLXGPUTuningServices), String> {
        let sysv = &*(*self.system).vtbl;
        let mut svc: *mut IADLXGPUTuningServices = std::ptr::null_mut();
        let r = (sysv.GetGPUTuningServices)(self.system, &mut svc);
        if r != ADLX_OK || svc.is_null() {
            return Err(format!("GetGPUTuningServices -> {}", result_name(r)));
        }
        let svcv = &*(*svc).vtbl;

        let mut supported: adlx_bool = 0;
        let r = (svcv.IsSupportedManualGFXTuning)(svc, gpu, &mut supported);
        if r != ADLX_OK {
            (svcv.Release)(svc);
            return Err(format!("IsSupportedManualGFXTuning -> {}", result_name(r)));
        }
        if supported == 0 {
            (svcv.Release)(svc);
            return Err("manual GFX tuning not supported on this GPU".into());
        }

        let mut ifc: *mut IADLXInterface = std::ptr::null_mut();
        let r = (svcv.GetManualGFXTuning)(svc, gpu, &mut ifc);
        if r != ADLX_OK || ifc.is_null() {
            (svcv.Release)(svc);
            return Err(format!("GetManualGFXTuning -> {}", result_name(r)));
        }

        // QueryInterface to the versioned (Navi) interface.
        let iid: Vec<u16> = "IADLXManualGraphicsTuning2"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ifcv = &*(*ifc).vtbl;
        let mut tuning2: *mut c_void = std::ptr::null_mut();
        let r = (ifcv.QueryInterface)(ifc, iid.as_ptr(), &mut tuning2);
        (ifcv.Release)(ifc);
        if r != ADLX_OK || tuning2.is_null() {
            (svcv.Release)(svc);
            return Err(format!(
                "QueryInterface(IADLXManualGraphicsTuning2) -> {} \
                 (GPU may be pre-Navi, which uses Tuning1)",
                result_name(r)
            ));
        }
        Ok((tuning2 as *mut IADLXManualGraphicsTuning2, svc))
    }

    /// Acquire the manual power tuning interface for a GPU. Returns None if not
    /// supported. Caller releases the returned pointer and the services ptr.
    unsafe fn manual_power(
        &self,
        gpu: *mut IADLXGPU,
    ) -> Result<Option<(*mut IADLXManualPowerTuning, *mut IADLXGPUTuningServices)>, String> {
        let sysv = &*(*self.system).vtbl;
        let mut svc: *mut IADLXGPUTuningServices = std::ptr::null_mut();
        let r = (sysv.GetGPUTuningServices)(self.system, &mut svc);
        if r != ADLX_OK || svc.is_null() {
            return Err(format!("GetGPUTuningServices -> {}", result_name(r)));
        }
        let svcv = &*(*svc).vtbl;

        let mut supported: adlx_bool = 0;
        let r = (svcv.IsSupportedManualPowerTuning)(svc, gpu, &mut supported);
        if r != ADLX_OK || supported == 0 {
            (svcv.Release)(svc);
            return Ok(None);
        }

        let mut ifc: *mut IADLXInterface = std::ptr::null_mut();
        let r = (svcv.GetManualPowerTuning)(svc, gpu, &mut ifc);
        if r != ADLX_OK || ifc.is_null() {
            (svcv.Release)(svc);
            return Err(format!("GetManualPowerTuning -> {}", result_name(r)));
        }
        let iid: Vec<u16> = "IADLXManualPowerTuning"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ifcv = &*(*ifc).vtbl;
        let mut power: *mut c_void = std::ptr::null_mut();
        let r = (ifcv.QueryInterface)(ifc, iid.as_ptr(), &mut power);
        (ifcv.Release)(ifc);
        if r != ADLX_OK || power.is_null() {
            (svcv.Release)(svc);
            return Err(format!(
                "QueryInterface(IADLXManualPowerTuning) -> {}",
                result_name(r)
            ));
        }
        Ok(Some((power as *mut IADLXManualPowerTuning, svc)))
    }

    /// Acquire the manual VRAM tuning interface (Navi/Tuning2) for a GPU.
    /// Returns None if not supported. Caller releases both pointers.
    unsafe fn manual_vram(
        &self,
        gpu: *mut IADLXGPU,
    ) -> Result<Option<(*mut IADLXManualVRAMTuning2, *mut IADLXGPUTuningServices)>, String> {
        let sysv = &*(*self.system).vtbl;
        let mut svc: *mut IADLXGPUTuningServices = std::ptr::null_mut();
        let r = (sysv.GetGPUTuningServices)(self.system, &mut svc);
        if r != ADLX_OK || svc.is_null() {
            return Err(format!("GetGPUTuningServices -> {}", result_name(r)));
        }
        let svcv = &*(*svc).vtbl;

        let mut supported: adlx_bool = 0;
        let r = (svcv.IsSupportedManualVRAMTuning)(svc, gpu, &mut supported);
        if r != ADLX_OK || supported == 0 {
            (svcv.Release)(svc);
            return Ok(None);
        }

        let mut ifc: *mut IADLXInterface = std::ptr::null_mut();
        let r = (svcv.GetManualVRAMTuning)(svc, gpu, &mut ifc);
        if r != ADLX_OK || ifc.is_null() {
            (svcv.Release)(svc);
            return Err(format!("GetManualVRAMTuning -> {}", result_name(r)));
        }
        let iid: Vec<u16> = "IADLXManualVRAMTuning2"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ifcv = &*(*ifc).vtbl;
        let mut vram: *mut c_void = std::ptr::null_mut();
        let r = (ifcv.QueryInterface)(ifc, iid.as_ptr(), &mut vram);
        (ifcv.Release)(ifc);
        if r != ADLX_OK || vram.is_null() {
            (svcv.Release)(svc);
            return Err(format!(
                "QueryInterface(IADLXManualVRAMTuning2) -> {} (pre-Navi GPU uses VRAMTuning1)",
                result_name(r)
            ));
        }
        Ok(Some((vram as *mut IADLXManualVRAMTuning2, svc)))
    }

    /// Acquire the manual fan tuning interface for a GPU. Returns None if not
    /// supported. Caller releases the returned pointer and the services ptr.
    unsafe fn manual_fan(
        &self,
        gpu: *mut IADLXGPU,
    ) -> Result<Option<(*mut IADLXManualFanTuning, *mut IADLXGPUTuningServices)>, String> {
        let sysv = &*(*self.system).vtbl;
        let mut svc: *mut IADLXGPUTuningServices = std::ptr::null_mut();
        let r = (sysv.GetGPUTuningServices)(self.system, &mut svc);
        if r != ADLX_OK || svc.is_null() {
            return Err(format!("GetGPUTuningServices -> {}", result_name(r)));
        }
        let svcv = &*(*svc).vtbl;

        let mut supported: adlx_bool = 0;
        let r = (svcv.IsSupportedManualFanTuning)(svc, gpu, &mut supported);
        if r != ADLX_OK || supported == 0 {
            (svcv.Release)(svc);
            return Ok(None);
        }

        let mut ifc: *mut IADLXInterface = std::ptr::null_mut();
        let r = (svcv.GetManualFanTuning)(svc, gpu, &mut ifc);
        if r != ADLX_OK || ifc.is_null() {
            (svcv.Release)(svc);
            return Err(format!("GetManualFanTuning -> {}", result_name(r)));
        }
        let iid: Vec<u16> = "IADLXManualFanTuning"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let ifcv = &*(*ifc).vtbl;
        let mut fan: *mut c_void = std::ptr::null_mut();
        let r = (ifcv.QueryInterface)(ifc, iid.as_ptr(), &mut fan);
        (ifcv.Release)(ifc);
        if r != ADLX_OK || fan.is_null() {
            (svcv.Release)(svc);
            return Err(format!(
                "QueryInterface(IADLXManualFanTuning) -> {}",
                result_name(r)
            ));
        }
        Ok(Some((fan as *mut IADLXManualFanTuning, svc)))
    }

    /// Apply a fan curve (temperature C, speed % points, low->high). Overwrites
    /// the GPU's current fan tuning states in order, validates, and commits.
    unsafe fn apply_fan_curve(&self, gpu: *mut IADLXGPU, curve: &[(i32, i32)]) -> String {
        let Some((fan, fsvc)) = (match self.manual_fan(gpu) {
            Ok(v) => v,
            Err(e) => return format!("fan curve: {e}"),
        }) else {
            return "fan curve: not supported on this GPU".into();
        };
        let fanv = &*(*fan).vtbl;
        let fsvcv = &*(*fsvc).vtbl;

        let mut speed_range = AdlxIntRange::default();
        let mut temp_range = AdlxIntRange::default();
        (fanv.GetFanTuningRanges)(fan, &mut speed_range, &mut temp_range);

        let mut list: *mut IADLXManualFanTuningStateList = std::ptr::null_mut();
        let r = (fanv.GetFanTuningStates)(fan, &mut list);
        let out = if r != ADLX_OK || list.is_null() {
            format!("fan curve: GetFanTuningStates -> {}", result_name(r))
        } else {
            let listv = &*(*list).vtbl;
            let size = (listv.Size)(list) as usize;
            let n = curve.len().min(size);
            for i in 0..n {
                let mut st: *mut IADLXManualFanTuningState = std::ptr::null_mut();
                if (listv.At_ManualFanTuningStateList)(list, i as adlx_uint, &mut st) == ADLX_OK
                    && !st.is_null()
                {
                    let stv = &*(*st).vtbl;
                    let (t, s) = curve[i];
                    (stv.SetTemperature)(st, clamp_simple(t, &temp_range));
                    (stv.SetFanSpeed)(st, clamp_simple(s, &speed_range));
                    (stv.Release)(st);
                }
            }
            let mut err_idx: adlx_int = -1;
            let rv = (fanv.IsValidFanTuningStates)(fan, list, &mut err_idx);
            let res = if rv == ADLX_OK && err_idx < 0 {
                let rs = (fanv.SetFanTuningStates)(fan, list);
                if rs == ADLX_OK {
                    format!("fan curve: applied {n} point(s) {curve:?}")
                } else {
                    format!("fan curve: SetFanTuningStates -> {}", result_name(rs))
                }
            } else {
                format!(
                    "fan curve: FAILED validation (errorIndex {err_idx}, ranges temp {}..{} speed {}..{})",
                    temp_range.min, temp_range.max, speed_range.min, speed_range.max
                )
            };
            (listv.Release)(list);
            res
        };

        (fanv.Release)(fan);
        (fsvcv.Release)(fsvc);
        out
    }

    /// Read the current tuning state (core clock, voltage, power).
    pub fn read_gfx_tuning(&self) -> Result<GfxTuning, String> {
        unsafe {
            let gpu = self.first_gpu()?;
            let gpuv = &*(*gpu).vtbl;
            let gpu_name = Self::gpu_string(gpu, gpuv.Name);
            let device_id = Self::gpu_string(gpu, gpuv.DeviceId);

            let (t2, svc) = self.manual_gfx2(gpu)?;
            let tv = &*(*t2).vtbl;
            let svcv = &*(*svc).vtbl;

            let mut min_range = AdlxIntRange::default();
            let mut max_range = AdlxIntRange::default();
            let mut volt_range = AdlxIntRange::default();
            let mut cur_min: adlx_int = 0;
            let mut cur_max: adlx_int = 0;
            let mut cur_volt: adlx_int = 0;
            (tv.GetGPUMinFrequencyRange)(t2, &mut min_range);
            (tv.GetGPUMaxFrequencyRange)(t2, &mut max_range);
            (tv.GetGPUVoltageRange)(t2, &mut volt_range);
            (tv.GetGPUMinFrequency)(t2, &mut cur_min);
            (tv.GetGPUMaxFrequency)(t2, &mut cur_max);
            (tv.GetGPUVoltage)(t2, &mut cur_volt);
            (tv.Release)(t2);
            (svcv.Release)(svc);

            let mut power_range = None;
            let mut cur_power = None;
            if let Some((pw, psvc)) = self.manual_power(gpu)? {
                let pwv = &*(*pw).vtbl;
                let psvcv = &*(*psvc).vtbl;
                let mut pr = AdlxIntRange::default();
                let mut cp: adlx_int = 0;
                (pwv.GetPowerLimitRange)(pw, &mut pr);
                (pwv.GetPowerLimit)(pw, &mut cp);
                power_range = Some(pr);
                cur_power = Some(cp);
                (pwv.Release)(pw);
                (psvcv.Release)(psvc);
            }

            let mut mem_range = None;
            let mut cur_mem = None;
            if let Some((vram, vsvc)) = self.manual_vram(gpu)? {
                let vv = &*(*vram).vtbl;
                let vsvcv = &*(*vsvc).vtbl;
                let mut mr = AdlxIntRange::default();
                let mut cm: adlx_int = 0;
                (vv.GetMaxVRAMFrequencyRange)(vram, &mut mr);
                (vv.GetMaxVRAMFrequency)(vram, &mut cm);
                mem_range = Some(mr);
                cur_mem = Some(cm);
                (vv.Release)(vram);
                (vsvcv.Release)(vsvc);
            }

            (gpuv.Release)(gpu);

            Ok(GfxTuning {
                gpu_name,
                device_id,
                min_range,
                max_range,
                volt_range,
                cur_min,
                cur_max,
                cur_volt,
                power_range,
                cur_power,
                mem_range,
                cur_mem,
            })
        }
    }

    /// Apply a set of tuning values. Each present field is clamped to its
    /// reported range and applied; absent fields are left untouched. Returns a
    /// per-field log of what was applied (read back from the GPU).
    pub fn apply(&self, set: &TuningSet) -> Result<Vec<String>, String> {
        let mut log = Vec::new();
        unsafe {
            let gpu = self.first_gpu()?;
            let gpuv = &*(*gpu).vtbl;

            // --- graphics tuning: max first (raise ceiling), then min, then voltage ---
            let (t2, svc) = self.manual_gfx2(gpu)?;
            let tv = &*(*t2).vtbl;
            let svcv = &*(*svc).vtbl;

            let mut min_range = AdlxIntRange::default();
            let mut max_range = AdlxIntRange::default();
            let mut volt_range = AdlxIntRange::default();
            (tv.GetGPUMinFrequencyRange)(t2, &mut min_range);
            (tv.GetGPUMaxFrequencyRange)(t2, &mut max_range);
            (tv.GetGPUVoltageRange)(t2, &mut volt_range);

            if let Some(req) = set.max_clock {
                let target = clamp_simple(req, &max_range);
                let r = (tv.SetGPUMaxFrequency)(t2, target);
                let mut got: adlx_int = 0;
                (tv.GetGPUMaxFrequency)(t2, &mut got);
                log.push(fmt_apply("max clock", req, target, got, r, &max_range));
            }
            if let Some(req) = set.min_clock {
                let mut cur_max: adlx_int = 0;
                (tv.GetGPUMaxFrequency)(t2, &mut cur_max);
                let target = clamp_to_range(req, &min_range, cur_max);
                let r = (tv.SetGPUMinFrequency)(t2, target);
                let mut got: adlx_int = 0;
                (tv.GetGPUMinFrequency)(t2, &mut got);
                log.push(fmt_apply("min clock", req, target, got, r, &min_range));
            }
            if let Some(req) = set.voltage {
                let target = clamp_simple(req, &volt_range);
                let r = (tv.SetGPUVoltage)(t2, target);
                let mut got: adlx_int = 0;
                (tv.GetGPUVoltage)(t2, &mut got);
                log.push(fmt_apply("voltage", req, target, got, r, &volt_range));
            }
            (tv.Release)(t2);
            (svcv.Release)(svc);

            // --- power tuning (separate interface) ---
            if let Some(req) = set.power_limit {
                match self.manual_power(gpu)? {
                    Some((pw, psvc)) => {
                        let pwv = &*(*pw).vtbl;
                        let psvcv = &*(*psvc).vtbl;
                        let mut pr = AdlxIntRange::default();
                        (pwv.GetPowerLimitRange)(pw, &mut pr);
                        let target = clamp_simple(req, &pr);
                        let r = (pwv.SetPowerLimit)(pw, target);
                        let mut got: adlx_int = 0;
                        (pwv.GetPowerLimit)(pw, &mut got);
                        log.push(fmt_apply("power limit", req, target, got, r, &pr));
                        (pwv.Release)(pw);
                        (psvcv.Release)(psvc);
                    }
                    None => log.push("power limit: not supported on this GPU".into()),
                }
            }

            // --- memory / VRAM clock (separate interface) ---
            if let Some(req) = set.mem_clock {
                match self.manual_vram(gpu)? {
                    Some((vram, vsvc)) => {
                        let vv = &*(*vram).vtbl;
                        let vsvcv = &*(*vsvc).vtbl;
                        let mut range = AdlxIntRange::default();
                        (vv.GetMaxVRAMFrequencyRange)(vram, &mut range);
                        let target = clamp_simple(req, &range);
                        let r = (vv.SetMaxVRAMFrequency)(vram, target);
                        let mut got: adlx_int = 0;
                        (vv.GetMaxVRAMFrequency)(vram, &mut got);
                        log.push(fmt_apply("mem clock", req, target, got, r, &range));
                        (vv.Release)(vram);
                        (vsvcv.Release)(vsvc);
                    }
                    None => log.push("mem clock: not supported on this GPU".into()),
                }
            }

            // --- fan curve (separate interface, state-list pattern) ---
            if let Some(curve) = &set.fan_curve {
                if curve.is_empty() {
                    log.push("fan curve: empty, skipped".into());
                } else {
                    log.push(self.apply_fan_curve(gpu, curve));
                }
            }

            (gpuv.Release)(gpu);
        }
        Ok(log)
    }

    /// Convenience: set only the GPU minimum core clock.
    pub fn set_gfx_min_clock(&self, requested: i32) -> Result<i32, String> {
        self.apply(&TuningSet {
            min_clock: Some(requested),
            ..Default::default()
        })?;
        Ok(self.read_gfx_tuning()?.cur_min)
    }

    /// Reset ALL GPU tuning (clock, voltage, power, fan) to factory defaults.
    pub fn reset_to_factory(&self) -> Result<(), String> {
        unsafe {
            let gpu = self.first_gpu()?;
            let gpuv = &*(*gpu).vtbl;
            let sysv = &*(*self.system).vtbl;
            let mut svc: *mut IADLXGPUTuningServices = std::ptr::null_mut();
            let r = (sysv.GetGPUTuningServices)(self.system, &mut svc);
            if r != ADLX_OK || svc.is_null() {
                (gpuv.Release)(gpu);
                return Err(format!("GetGPUTuningServices -> {}", result_name(r)));
            }
            let svcv = &*(*svc).vtbl;
            let r = (svcv.ResetToFactory)(svc, gpu);
            (svcv.Release)(svc);
            (gpuv.Release)(gpu);
            if r != ADLX_OK {
                return Err(format!("ResetToFactory -> {}", result_name(r)));
            }
            Ok(())
        }
    }
}

fn fmt_apply(
    label: &str,
    requested: i32,
    target: i32,
    got: i32,
    r: ADLX_RESULT,
    range: &AdlxIntRange,
) -> String {
    if r == ADLX_OK {
        let clamp_note = if target != requested {
            format!(" (requested {requested}, clamped to range {}..{})", range.min, range.max)
        } else {
            String::new()
        };
        format!("{label}: set {target} -> readback {got}{clamp_note}")
    } else {
        format!(
            "{label}: FAILED {} (requested {requested}, target {target}, range {}..{} step {})",
            result_name(r),
            range.min,
            range.max,
            range.step
        )
    }
}

impl Drop for Adlx {
    fn drop(&mut self) {
        unsafe {
            (self.terminate)();
        }
    }
}

/// Clamp to [min,max] and round down to a valid step offset from min.
fn clamp_simple(requested: i32, range: &AdlxIntRange) -> i32 {
    let step = range.step.max(1);
    let v = requested.clamp(range.min, range.max);
    (range.min + ((v - range.min) / step) * step).clamp(range.min, range.max)
}

fn clamp_to_range(requested: i32, range: &AdlxIntRange, cur_max: i32) -> i32 {
    let step = range.step.max(1);
    // upper bound: range max, and must stay below the current max clock
    let hi = range.max.min(cur_max);
    let mut v = requested.clamp(range.min, hi);
    // round down to a valid step offset from range.min
    v = range.min + ((v - range.min) / step) * step;
    v.clamp(range.min, hi)
}
