package osapi

import (
	"fmt"
	"syscall"
	"unsafe"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
	"golang.org/x/sys/windows"
)

func EnumWindowsCallback(hwnd uintptr, _ uintptr) uintptr {
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
	for _, activeWindow := range activeWindows {
		if activeWindow.Handle == hwnd {
			window.OriginalRect = activeWindow.OriginalRect
			break
		}
	}
	if window.OriginalRect == (RECT{}) {
		window.OriginalRect = GetWindowRect(hwnd)
	}
	window.MonitorInfo = GetMonitorByWindow(hwnd)

	exePath, err := GetProcessImagePath(window.Process)
	if err != nil {
		ErrorLog(fmt.Sprintf("getProcessImagePath failed for PID %d: %v", window.Process, err))
	} else {
		window.Executable = exePath
	}

	if window.Executable != "" {
		if desc, ferr := GetFileDescriptionByPath(window.Executable); ferr != nil {
			ErrorLog(fmt.Sprintf("getFileDescriptionByPath(%q) failed: %v", window.Executable, ferr))
		} else if desc != "" {
			window.Title = desc
		}
	}

	activeWindows = append(activeWindows, window)
	return 1
}
func GetProcessImagePath(pid uint32) (path string, err error) {
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
func GetFileDescriptionByPath(path string) (desc string, err error) {
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
		return QueryFileDescription(buf, 0x0409, 0x04B0)
	}

	start := uintptr(unsafe.Pointer(&buf[0]))
	offset := transPtr - start
	lang := *(*uint16)(unsafe.Pointer(&buf[offset]))
	codepage := *(*uint16)(unsafe.Pointer(&buf[offset+2]))

	return QueryFileDescription(buf, lang, codepage)
}
func QueryFileDescription(buf []byte, lang, codepage uint16) (desc string, err error) {
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
	return desc, nil
}
func GetMonitorByWindow(hwnd uintptr) RECT {
	r0, _, _ := procMonitorFromWindow.Call(hwnd, uintptr(MONITOR_DEFAULTTONEAREST))
	if r0 == 0 {
		ErrorLog("failed to get monitor for window")
	}
	var mi MONITORINFO
	mi.CbSize = uint32(unsafe.Sizeof(MONITORINFO{}))

	var rect RECT
	rect.Left = mi.RcMonitor.Left
	rect.Top = mi.RcMonitor.Top
	rect.Right = mi.RcMonitor.Right
	rect.Bottom = mi.RcMonitor.Bottom
	return rect
}
func SetWindowPos(hwnd uintptr, hwndInsertAfter uintptr, x, y, cx, cy int32, flags uint32) bool {
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
