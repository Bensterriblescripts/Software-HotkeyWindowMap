package main

import (
	"fmt"
	"syscall"
	"unsafe"

	"golang.org/x/sys/windows"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

type HWNDManager struct{}

var (
	user32   = windows.NewLazySystemDLL("user32.dll")
	version  = windows.NewLazySystemDLL("version.dll")
	kernel32 = windows.NewLazySystemDLL("kernel32.dll")

	procEnumWindows = user32.NewProc("EnumWindows")

	procGetWindowText       = user32.NewProc("GetWindowTextW")
	procGetWindowTextLength = user32.NewProc("GetWindowTextLengthW")

	procIsWindowVisible            = user32.NewProc("IsWindowVisible")
	procGetWindowThreadProcessId   = user32.NewProc("GetWindowThreadProcessId")
	procQueryFullProcessImageNameW = kernel32.NewProc("QueryFullProcessImageNameW")

	procGetFileVersionInfoSizeW = version.NewProc("GetFileVersionInfoSizeW")
	procGetFileVersionInfoW     = version.NewProc("GetFileVersionInfoW")
	procVerQueryValueW          = version.NewProc("VerQueryValueW")
)

type Window struct {
	Title      string
	FullTitle  string
	Handle     uintptr
	Process    uint32
	Executable string
}

var activeWindows []Window

func enumWindowsCallback(hwnd uintptr, _ uintptr) uintptr {
	// Visible windows only
	if visible, _, _ := procIsWindowVisible.Call(hwnd); visible == 0 {
		return 1
	}

	var window Window
	window.Handle = hwnd

	// Process ID
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

	// Window title
	window.Title = getWindowTitleByHandle(hwnd)
	if window.Title == "" {
		return 1
	}
	window.FullTitle = window.Title

	// Executable path
	exePath, err := getProcessImagePath(window.Process)
	if err != nil {
		ErrorLog(fmt.Sprintf("getProcessImagePath failed for PID %d: %v", window.Process, err))
	} else {
		window.Executable = exePath
	}

	// Description from file version info if we have a path
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

// getProcessImagePath returns the full path of the process executable.
func getProcessImagePath(pid uint32) (path string, err error) {

	// Needs PROCESS_QUERY_LIMITED_INFORMATION – works for 64-bit targets enumerating 64-bit processes
	const PROCESS_QUERY_LIMITED_INFORMATION = 0x1000

	h, err := windows.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
	if err != nil {
		return "", err
	}

	// Make sure we always close the handle, and log if CloseHandle fails.
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

// getFileDescriptionByPath attempts to read the "FileDescription" from the file's version info.
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
		// No version info – not necessarily an error
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
		// Fallback to US English / Unicode
		return queryFileDescription(buf, 0x0409, 0x04B0)
	}

	lang := *(*uint16)(unsafe.Pointer(transPtr))
	codepage := *(*uint16)(unsafe.Pointer(transPtr + unsafe.Sizeof(lang)))

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

	desc = windows.UTF16PtrToString((*uint16)(unsafe.Pointer(valuePtr)))
	TraceLog(fmt.Sprintf("File description (%s): %q", subBlock, desc))
	return desc, nil
}

func getWindowTitleByHandle(hwnd uintptr) string {
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

func (h *HWNDManager) GetAllActiveWindows() []Window {
	TraceLog("GetAllActiveWindows started")
	activeWindows = make([]Window, 0)

	cb := syscall.NewCallback(enumWindowsCallback)
	ret, _, err := procEnumWindows.Call(
		cb,
		0,
	)

	// EnumWindows returns non-zero on success.
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
