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
	SW_SHOWMAXIMIZED         = 3
	WS_OVERLAPPEDWINDOW      = WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU
)

/* Window Information and Retrieval */
func FindWindowByTitle(title string) uintptr {
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

	cb := syscall.NewCallback(enumWindowsCallback)
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

type Window struct {
	Title       string
	FullTitle   string
	Handle      uintptr
	Process     uint32
	Executable  string
	WindowState string
}

var activeWindows []Window

/* Helper Functions */
func enumWindowsCallback(hwnd uintptr, _ uintptr) uintptr {
	if visible, _, _ := procIsWindowVisible.Call(hwnd); visible == 0 {
		return 1
	}

	var window Window
	window.Handle = hwnd
	if _, _, e := procGetWindowThreadProcessId.Call(
		hwnd,
		uintptr(unsafe.Pointer(&window.Process)),
	); window.Process == 0 {
		if e != nil && e != syscall.Errno(0) {
			ErrorLog(fmt.Sprintf("GetWindowThreadProcessId failed for hwnd=0x%x: %v", hwnd, e))
		} else {
			TraceLog(fmt.Sprintf("Skipping window with PID 0: hwnd=0x%x", hwnd))
		}
		return 1
	}

	window.Title = GetWindowTitle(hwnd)
	if window.Title == "" {
		return 1
	}
	window.FullTitle = window.Title

	exePath, err := getProcessImagePath(window.Process)
	if err != nil {
		ErrorLog(fmt.Sprintf("getProcessImagePath failed for PID %d: %v", window.Process, err))
	} else {
		window.Executable = exePath
	}

	if window.Executable != "" {
		if desc, ferr := getFileDescriptionByPath(window.Executable); ferr != nil {
			ErrorLog(fmt.Sprintf("getFileDescriptionByPath(%q) failed: %v", window.Executable, ferr))
		} else if desc != "" {
			window.Title = desc
		}
	}

	TraceLog(fmt.Sprintf("Found Window: hwnd=0x%x, title=%q", hwnd, window.Title))
	activeWindows = append(activeWindows, window)
	return 1
}
func getProcessImagePath(pid uint32) (path string, err error) {
	const PROCESS_QUERY_LIMITED_INFORMATION = 0x1000

	h, err := windows.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
	if err != nil {
		return "", err
	}

	defer func() {
		err := windows.CloseHandle(h)
		if err != nil {
			ErrorLog(fmt.Sprintf("CloseHandle failed for pid=%d: %v", pid, err))
		}
	}()

	buf := make([]uint16, 260)
	size := uint32(len(buf))

	r0, _, e := procQueryFullProcessImageNameW.Call(
		uintptr(h),
		0,
		uintptr(unsafe.Pointer(&buf[0])),
		uintptr(unsafe.Pointer(&size)),
	)
	if r0 == 0 {
		if e != nil && e != syscall.Errno(0) {
			err = e
			return "", err
		}
		err = fmt.Errorf("QueryFullProcessImageNameW returned 0 without extended error")
		return "", err
	}

	path = windows.UTF16ToString(buf[:size])
	return path, nil
}
func getFileDescriptionByPath(path string) (desc string, err error) {
	if path == "" {
		return "", nil
	}

	p, err := syscall.UTF16PtrFromString(path)
	if err != nil {
		return "", err
	}

	var handle uint32
	r0, _, callErr := procGetFileVersionInfoSizeW.Call(
		uintptr(unsafe.Pointer(p)),
		uintptr(unsafe.Pointer(&handle)),
	)
	if r0 == 0 {
		TraceLog(fmt.Sprintf("No version info available for %q", path))
		return "", nil
	}
	size := uint32(r0)

	buf := make([]byte, size)
	r0, _, callErr = procGetFileVersionInfoW.Call(
		uintptr(unsafe.Pointer(p)),
		0,
		uintptr(size),
		uintptr(unsafe.Pointer(&buf[0])),
	)
	if r0 == 0 {
		if callErr != nil && callErr != syscall.Errno(0) {
			err = callErr
			return "", err
		}
		err = fmt.Errorf("GetFileVersionInfoW returned 0 for %q", path)
		return "", err
	}

	var transPtr uintptr
	var transLen uint32
	r0, _, callErr = procVerQueryValueW.Call(
		uintptr(unsafe.Pointer(&buf[0])),
		uintptr(unsafe.Pointer(syscall.StringToUTF16Ptr(`\VarFileInfo\Translation`))),
		uintptr(unsafe.Pointer(&transPtr)),
		uintptr(unsafe.Pointer(&transLen)),
	)
	if r0 == 0 || transLen < 4 {
		if callErr != nil && callErr != syscall.Errno(0) {
			TraceLog(fmt.Sprintf("VerQueryValueW(Translation) fallback for %q: %v", path, callErr))
		}
		return queryFileDescription(buf, 0x0409, 0x04B0)
	}

	start := uintptr(unsafe.Pointer(&buf[0]))
	offset := transPtr - start
	lang := *(*uint16)(unsafe.Pointer(&buf[offset]))
	codepage := *(*uint16)(unsafe.Pointer(&buf[offset+2]))

	return queryFileDescription(buf, lang, codepage)
}
func queryFileDescription(buf []byte, lang, codepage uint16) (desc string, err error) {
	subBlock := fmt.Sprintf(`\StringFileInfo\%04x%04x\FileDescription`, lang, codepage)

	var valuePtr uintptr
	var valueLen uint32
	r0, _, callErr := procVerQueryValueW.Call(
		uintptr(unsafe.Pointer(&buf[0])),
		uintptr(unsafe.Pointer(syscall.StringToUTF16Ptr(subBlock))),
		uintptr(unsafe.Pointer(&valuePtr)),
		uintptr(unsafe.Pointer(&valueLen)),
	)
	if r0 == 0 || valueLen == 0 {
		if callErr != nil && callErr != syscall.Errno(0) {
			err = callErr
			return "", err
		}
		err = fmt.Errorf("VerQueryValueW returned no data for %q", subBlock)
		return "", err
	}

	start := uintptr(unsafe.Pointer(&buf[0]))
	offset := valuePtr - start
	desc = windows.UTF16PtrToString((*uint16)(unsafe.Pointer(&buf[int(offset)])))
	TraceLog(fmt.Sprintf("File description (%s): %q", subBlock, desc))
	return desc, nil
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

/* Alter Window State and Focus */
func SetBorderlessWindow(hwnd uintptr) {
	x, y, width, height := getMonitorByWindow(hwnd)
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
	setWindowPos(
		hwnd,
		0,
		x,
		y,
		width,
		height,
		SWP_FRAMECHANGED|SWP_SHOWWINDOW,
	)

	procShowWindow.Call(
		hwnd,
		uintptr(SW_SHOW),
	)
	for _, window := range activeWindows {
		if window.Handle == hwnd {
			window.WindowState = "Borderless"
			break
		}
	}

	SetFocus(hwnd)
}
func SetWindowWindowed(hwnd uintptr) {
	x, y, width, height := getMonitorByWindow(hwnd)

	styleIndex := int32(GWL_STYLE)
	r2, _, _ := procGetWindowLongW.Call(hwnd, uintptr(styleIndex))
	origStyle := r2

	newStyle := (origStyle | uintptr(WS_OVERLAPPEDWINDOW)) &^ uintptr(WS_POPUP)

	procSetWindowLongW.Call(hwnd, uintptr(styleIndex), newStyle)

	setWindowPos(
		hwnd,
		0,
		x,
		y,
		width,
		height,
		SWP_FRAMECHANGED|SWP_SHOWWINDOW,
	)

	procShowWindow.Call(hwnd, uintptr(SW_SHOW))

	for _, window := range activeWindows {
		if window.Handle == hwnd {
			window.WindowState = "Windowed"
			break
		}
	}

	SetFocus(hwnd)
}
func SetFocus(hwnd uintptr) {
	if hwnd == 0 {
		ErrorLog("SetFocus: window handle is null")
		return
	}
	procShowWindow.Call(hwnd, uintptr(SW_SHOWMAXIMIZED))

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
func getMonitorByWindow(hwnd uintptr) (x int32, y int32, width int32, height int32) {
	r0, _, _ := procMonitorFromWindow.Call(hwnd, uintptr(MONITOR_DEFAULTTONEAREST))
	if r0 == 0 {
		ErrorLog("failed to get monitor for window")
	}
	monitor := r0
	mi := MONITORINFO{
		CbSize: uint32(unsafe.Sizeof(MONITORINFO{})),
	}
	r1, _, _ := procGetMonitorInfoW.Call(monitor, uintptr(unsafe.Pointer(&mi)))
	if r1 == 0 {
		ErrorLog("GetMonitorInfoW failed")
	}
	x = mi.RcMonitor.Left
	y = mi.RcMonitor.Top
	width = mi.RcMonitor.Right - mi.RcMonitor.Left
	height = mi.RcMonitor.Bottom - mi.RcMonitor.Top
	return x, y, width, height
}
func setWindowPos(hwnd uintptr, hwndInsertAfter uintptr, x, y, cx, cy int32, flags uint32) bool {
	r, _, _ := procSetWindowPos.Call(
		uintptr(hwnd),
		uintptr(hwndInsertAfter),
		uintptr(x),
		uintptr(y),
		uintptr(cx),
		uintptr(cy),
		uintptr(flags),
	)
	return r != 0
}
