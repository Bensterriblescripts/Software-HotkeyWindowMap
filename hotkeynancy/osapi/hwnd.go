package osapi

import (
	"fmt"
	"syscall"
	"unsafe"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
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
	WS_OVERLAPPEDWINDOW      = WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU

	WM_HOTKEY    = 0x0312
	MOD_ALT      = 0x0001
	MOD_CONTROL  = 0x0002
	MOD_SHIFT    = 0x0004
	MOD_WIN      = 0x0008
	MOD_NOREPEAT = 0x4000
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

var activeWindows []Window

/* Get Information and Retrieval */
func GetWindowByTitle(title string) uintptr {
	t, err := windows.UTF16PtrFromString(title)
	if err != nil {
		return 0
	}
	r, _, _ := procFindWindowW.Call(
		0,
		uintptr(unsafe.Pointer(t)),
	)

	return r
}
func GetWindowTitle(hwnd uintptr) string {
	ret, _, callErr := procGetWindowTextLength.Call(hwnd)
	length := uint32(ret)
	if length == 0 {
		if callErr != nil && callErr != syscall.Errno(0) {
			TraceLog(fmt.Sprintf("GetWindowTextLength failed for hwnd=0x%x: %v", hwnd, callErr))
		}
		return ""
	}
	buf := make([]uint16, length+1)

	_, _, callErr = procGetWindowText.Call(
		hwnd,
		uintptr(unsafe.Pointer(&buf[0])),
		uintptr(length+1),
	)
	if callErr != nil && callErr != syscall.Errno(0) {
		TraceLog(fmt.Sprintf("GetWindowText failed for hwnd=0x%x: %v", hwnd, callErr))
	}

	return windows.UTF16ToString(buf)
}
func GetAllActiveWindows() []Window {
	TraceLog("GetAllActiveWindows started")
	activeWindows = make([]Window, 0)

	cb := syscall.NewCallback(EnumWindowsCallback)
	ret, _, err := procEnumWindows.Call(
		cb,
		0,
	)

	if ret == 0 {
		if err != nil && err != syscall.Errno(0) {
			ErrorLog("Error while iterating through windows: " + err.Error())
		} else {
			ErrorLog("EnumWindows returned 0 without extended error")
		}
		return nil
	}

	TraceLog(fmt.Sprintf("GetAllActiveWindows finished, found %d windows", len(activeWindows)))
	return activeWindows
}
func GetScreenSize() (width, height int32) {
	w := GetSystemMetrics(SM_CXSCREEN)
	h := GetSystemMetrics(SM_CYSCREEN)
	return w, h
}
func GetSystemMetrics(index int32) int32 {
	r, _, _ := procGetSystemMetrics.Call(uintptr(index))
	return int32(r)
}
func GetWindowState(hwnd uintptr) string {
	styleIndex := int32(GWL_STYLE)
	style, _, _ := procGetWindowLongW.Call(hwnd, uintptr(styleIndex))
	if style&uintptr(WS_OVERLAPPEDWINDOW) == 0 && style&uintptr(WS_POPUP) == uintptr(WS_POPUP) { // WARNING: Will false-positive most borderless applications that aren't applied by this program
		return "Fullscreen"
	} else if style&uintptr(WS_OVERLAPPEDWINDOW) == 0 {
		return "Borderless"
	} else {
		return "Windowed"
	}
}
func GetWindowRect(hwnd uintptr) RECT {
	var rect RECT
	r, _, _ := procGetWindowRect.Call(hwnd, uintptr(unsafe.Pointer(&rect)), uintptr(unsafe.Sizeof(rect)))
	if r == 0 {
		ErrorLog("GetWindowRect failed")
		return RECT{}
	}
	return rect
}

/* Alter Window State and Focus */
func SetBorderlessWindow(hwnd uintptr) {
	var window Window
	for _, activeWindow := range activeWindows {
		if activeWindow.Handle == hwnd {
			window = activeWindow
			break
		}
	}
	if window.Handle == 0 {
		TraceLog("Window not found, refreshing active windows...")
		GetAllActiveWindows()
		for _, activeWindow := range activeWindows {
			if activeWindow.Handle == hwnd {
				window = activeWindow
				break
			}
		}
		if window.Handle == 0 {
			ErrorLog("Tried to edit a handle that no longer exists")
			return
		}
	}
	window.OriginalRect = GetWindowRect(hwnd)

	styleIndex := int32(GWL_STYLE)
	origStyle, _, callErr := procGetWindowLongW.Call(
		hwnd,
		uintptr(styleIndex),
	)
	if origStyle == 0 && callErr != nil && callErr != syscall.Errno(0) {
		ErrorLog(fmt.Sprintf("GetWindowLongW(GWL_STYLE) failed for hwnd=0x%x: %v", hwnd, callErr))
		return
	}

	newStyle := origStyle &^ uintptr(WS_OVERLAPPEDWINDOW)

	r2, _, callErr := procSetWindowLongW.Call(
		hwnd,
		uintptr(styleIndex),
		newStyle,
	)
	if r2 == 0 && callErr != nil && callErr != syscall.Errno(0) {
		ErrorLog(fmt.Sprintf("SetWindowLongW(GWL_STYLE) failed for hwnd=0x%x: %v", hwnd, callErr))
		return
	}

	SetWindowPos(
		hwnd,
		0,
		window.MonitorInfo.Left,
		window.MonitorInfo.Top,
		window.MonitorInfo.Right-window.MonitorInfo.Left,
		window.MonitorInfo.Bottom-window.MonitorInfo.Top,
		SWP_FRAMECHANGED|SWP_SHOWWINDOW,
	)

	procShowWindow.Call(
		hwnd,
		uintptr(SW_SHOW),
	)

	window.WindowState = "Borderless"
	SetVisible(hwnd)
}
func SetWindowWindowed(hwnd uintptr) {
	var window Window
	for _, activeWindow := range activeWindows {
		if activeWindow.Handle == hwnd {
			window = activeWindow
			break
		}
	}
	if window.Handle == 0 {
		TraceLog("Window not found, refreshing active windows...")
		GetAllActiveWindows()
		for _, activeWindow := range activeWindows {
			if activeWindow.Handle == hwnd {
				window = activeWindow
				break
			}
		}
		if window.Handle == 0 {
			ErrorLog("Tried to edit a handle that no longer exists")
			return
		}
	}

	styleIndex := int32(GWL_STYLE)
	r2, _, _ := procGetWindowLongW.Call(hwnd, uintptr(styleIndex))
	origStyle := r2

	newStyle := (origStyle | uintptr(WS_OVERLAPPEDWINDOW)) &^ uintptr(WS_POPUP)
	procSetWindowLongW.Call(hwnd, uintptr(styleIndex), newStyle)

	SetWindowPos(
		hwnd,
		0,
		window.OriginalRect.Left,
		window.OriginalRect.Top,
		window.OriginalRect.Right-window.OriginalRect.Left,
		window.OriginalRect.Bottom-window.OriginalRect.Top,
		SWP_FRAMECHANGED|SWP_SHOWWINDOW,
	)

	procShowWindow.Call(hwnd, uintptr(SW_SHOW))

	window.WindowState = "Windowed"
	SetVisible(hwnd)
}
func SetFocus(hwnd uintptr) { // Bring window to front and steal focus from other windows
	if hwnd == 0 {
		ErrorLog("SetFocus: window handle is null")
		return
	}

	r2, _, _ := procBringWindowToTop.Call(hwnd)
	if r2 == 0 {
		ErrorLog("SetFocus: failed to bring window to top")
		return
	}

	r3, _, _ := procSetForegroundWindow.Call(hwnd)
	if r3 == 0 {
		ErrorLog("SetFocus: failed to set foreground window")
		return
	}
}
func SetVisible(hwnd uintptr) { // Less aggressive than SetFocus, will open if minimised/tray
	procShowWindow.Call(hwnd, SW_RESTORE)
}
func GetMessage(msg *MSG, hwnd uintptr, msgFilterMin uint32, msgFilterMax uint32) int32 {
	r, _, _ := procGetMessageW.Call(
		uintptr(unsafe.Pointer(msg)),
		hwnd,
		uintptr(msgFilterMin),
		uintptr(msgFilterMax),
	)
	return int32(r)
}
