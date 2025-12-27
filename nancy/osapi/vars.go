package osapi

import (
	"golang.org/x/sys/windows"
)

var (
	user32   = windows.NewLazySystemDLL("user32.dll")
	kernel32 = windows.NewLazySystemDLL("kernel32.dll")
	version  = windows.NewLazySystemDLL("version.dll")

	procEnumWindows = user32.NewProc("EnumWindows")

	procGetWindowText       = user32.NewProc("GetWindowTextW")
	procGetWindowTextLength = user32.NewProc("GetWindowTextLengthW")
	procIsWindowVisible     = user32.NewProc("IsWindowVisible")
	procFindWindowW         = user32.NewProc("FindWindowW")
	procGetSystemMetrics    = user32.NewProc("GetSystemMetrics")
	procGetWindowLongW      = user32.NewProc("GetWindowLongW")
	procSetWindowLongW      = user32.NewProc("SetWindowLongW")
	procGetWindowRect       = user32.NewProc("GetWindowRect")

	procMonitorFromWindow = user32.NewProc("MonitorFromWindow")
	procGetMonitorInfoW   = user32.NewProc("GetMonitorInfoW")
	procShowWindow        = user32.NewProc("ShowWindow")

	procGetWindowThreadProcessId   = user32.NewProc("GetWindowThreadProcessId")
	procQueryFullProcessImageNameW = kernel32.NewProc("QueryFullProcessImageNameW")

	procGetFileVersionInfoSizeW = version.NewProc("GetFileVersionInfoSizeW")
	procGetFileVersionInfoW     = version.NewProc("GetFileVersionInfoW")
	procVerQueryValueW          = version.NewProc("VerQueryValueW")

	procSetWindowPos = user32.NewProc("SetWindowPos")

	procBringWindowToTop    = user32.NewProc("BringWindowToTop")
	procSetForegroundWindow = user32.NewProc("SetForegroundWindow")
	procGetMessageW         = user32.NewProc("GetMessageW")

	procRegisterHotKey   = user32.NewProc("RegisterHotKey")
	procUnregisterHotKey = user32.NewProc("UnregisterHotKey")
)

const (
	SM_CXSCREEN            = 0 // width of primary monitor
	SM_CYSCREEN            = 1 // height of primary monitor
	SWP_FRAMECHANGED       = 0x0020
	SWP_SHOWWINDOW         = 0x0040
	GWL_STYLE        int32 = -16

	WS_POPUP       = 0x80000000
	WS_CAPTION     = 0x00C00000
	WS_THICKFRAME  = 0x00040000
	WS_MINIMIZEBOX = 0x00020000
	WS_MAXIMIZEBOX = 0x00010000
	WS_SYSMENU     = 0x00080000

	MONITOR_DEFAULTTONEAREST = 0x00000002
	SW_SHOW                  = 5
	SW_RESTORE               = 9
	SW_SHOWMAXIMIZED         = 3
	SW_MINIMIZE              = 0x20000000
	WS_OVERLAPPEDWINDOW      = WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU
)

type Window struct {
	Title        string
	FullTitle    string
	Handle       uintptr
	Process      uint32
	Executable   string
	WindowState  string
	OriginalRect RECT
	MonitorInfo  RECT
}
type RECT struct {
	Left   int32
	Top    int32
	Right  int32
	Bottom int32
}
type MONITORINFO struct {
	CbSize    uint32
	RcMonitor RECT
	RcWork    RECT
	DwFlags   uint32
}

type MSG struct {
	Hwnd     uintptr
	Message  uint32
	WParam   uintptr
	LParam   uintptr
	Time     uint32
	Pt       POINT
	LPrivate uint32
}

type POINT struct {
	X int32
	Y int32
}
