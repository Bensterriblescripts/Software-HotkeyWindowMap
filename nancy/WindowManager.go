package main

import (
	"nancy/osapi"

	"github.com/Bensterriblescripts/Lib-Handlers/config"
	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

type WindowManager struct{}

var activeWindows []osapi.Window
var activeHotkeys map[string][2]string
var hotkeyConfig map[string]string

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
func (h *WindowManager) SetFocus(handle int) {
	osapi.SetFocus(uintptr(handle))
}

func (h *WindowManager) GetAllHotkeys() map[string][2]string {
	return activeHotkeys
}
func (h *WindowManager) SetHotkey(executable string, kotkeymod string, hotkey string) {
	activeHotkeys[executable] = [2]string{kotkeymod, hotkey}

	hotkeyConfig = make(map[string]string, len(activeHotkeys)+1)
	for exe, keys := range activeHotkeys {
		hotkeyConfig[exe] = keys[0] + "+" + keys[1]
	}
	config.WriteSettings(hotkeyConfig)
}
