package main

import (
	"hotkeynancy/osapi"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

type WindowManager struct{}

var activeWindows []osapi.Window
var activeHotkeys map[int]string
var ignoredWindows map[string]bool

func (h *WindowManager) GetAllActiveWindows() []osapi.Window {
	activeWindows = []osapi.Window{}
	allWindows := osapi.GetAllActiveWindows()
	for _, window := range allWindows {
		window.WindowState = osapi.GetWindowState(window.Handle)
		activeWindows = append(activeWindows, window)
	}
	return activeWindows
}

func (h *WindowManager) SetBorderlessFullscreen(handle int) {
	for index, window := range activeWindows {
		if window.Handle == uintptr(handle) {
			if window.WindowState == "Borderless" {
				TraceLog("Window is already borderless")
				return
			} else {
				activeWindows[index].WindowState = "Borderless"
				break
			}
		}
	}
	osapi.SetBorderlessWindow(uintptr(handle))
}
func (h *WindowManager) SetWindowed(handle int) {
	for index, window := range activeWindows {
		if window.Handle == uintptr(handle) {
			if window.WindowState == "Windowed" {
				TraceLog("Window is already windowed")
				return
			} else {
				activeWindows[index].WindowState = "Windowed"
				break
			}
		}
	}
	osapi.SetWindowWindowed(uintptr(handle))
}

func (h *WindowManager) GetAllHotkeys() map[int]string {
	return activeHotkeys
}
func (h *WindowManager) SetHotkey(handle int, hotkey string) {
}
