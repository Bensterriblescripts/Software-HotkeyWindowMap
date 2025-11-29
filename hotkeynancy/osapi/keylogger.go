package osapi

import (
	"runtime"
	"time"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

var LogKeys = true

const (
	VK_F1    = 0x70
	hotkeyID = 1
)

func StartKeylogger() {
	go func() {
		runtime.LockOSThread()
		defer runtime.UnlockOSThread()

		if !RegisterHotKey(0, hotkeyID, MOD_ALT, VK_F1) {
			ErrorLog("RegisterHotKey(ALT+F1) failed")
			return
		}
		defer UnregisterHotKey(0, hotkeyID)

		for LogKeys {
			msg := MSG{}
			if GetMessage(&msg, 0, 0, 0) <= 0 {
				break
			}
			if msg.Message == WM_HOTKEY && msg.WParam == uintptr(hotkeyID) {
				TraceLog("ALT+F1 hotkey pressed")
			}

			time.Sleep(100 * time.Millisecond)
		}
	}()

	go func() {
		for {
			PrintProcUsage()
			time.Sleep(1000 * time.Millisecond)
		}
	}()
}

func RegisterHotKey(hwnd uintptr, id int32, modifiers uint32, vk uint32) bool {
	r, _, _ := procRegisterHotKey.Call(
		hwnd,
		uintptr(id),
		uintptr(modifiers),
		uintptr(vk),
	)
	return r != 0
}

func UnregisterHotKey(hwnd uintptr, id int32) bool {
	r, _, _ := procUnregisterHotKey.Call(
		hwnd,
		uintptr(id),
	)
	return r != 0
}
