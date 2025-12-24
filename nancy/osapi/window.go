package osapi

import (
	"fmt"
	"syscall"
	"unsafe"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
	"golang.org/x/sys/windows"
)

var activeWindows []Window

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
func SetWindowMinimised(hwnd uintptr) {
	if hwnd == 0 {
		ErrorLog("Passed in an empty pointer, did not minimise window")
		return
	}

	var window Window
	for _, activeWindow := range activeWindows {
		if activeWindow.Handle == hwnd {
			window = activeWindow
			break
		}
	}
	if window.Handle == 0 {
		TraceLog("Window was not found in active window array, retrieving list...")
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

	procShowWindow.Call(hwnd, uintptr(SW_MINIMIZE))

	window.WindowState = "Minimised"
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
