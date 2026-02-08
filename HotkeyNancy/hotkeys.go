package main

import (
	"fmt"

	"github.com/Bensterriblescripts/Lib-Handlers/config"
	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
	"github.com/Bensterriblescripts/Lib-Handlers/osapi"
)

type HotkeyService struct{}

var activeWindows []osapi.Window
var activeHotkeys map[string][2]string
var hotkeyConfig map[string]string

func (h *HotkeyService) GetAllActiveWindows() []osapi.Window {
	activeWindows = []osapi.Window{}
	allWindows := osapi.GetAllActiveWindows()
	for _, window := range allWindows {
		if window.Title == "Windows Explorer" || window.Title == "Settings" || window.Title == "Application Frame Host" || window.Title == "Windows Input Experience" {
			continue
		}
		window.WindowState = osapi.GetWindowState(window.Handle)
		if window.WindowState == "Borderless" {
			currentRect := osapi.GetWindowRect(window.Handle)
			if currentRect == window.OriginalRect {
				window.WindowState = "Borderless (Via Application)"
			}
		}
		activeWindows = append(activeWindows, window)
	}
	return activeWindows
}

func (h *HotkeyService) SetBorderlessFullscreen(handle int) {
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
func (h *HotkeyService) SetWindowed(handle int) {
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
func (h *HotkeyService) SetFocus(handle int) {
	osapi.SetFocus(uintptr(handle))
}

func (h *HotkeyService) GetAllHotkeys() map[string][2]string {
	activeHotkeys = make(map[string][2]string)
	return activeHotkeys
}
func (h *HotkeyService) SetHotkey(executable string, kotkeymod string, hotkey string) {
	if activeHotkeys[executable] == [2]string{} {
		activeHotkeys = make(map[string][2]string)
	}
	activeHotkeys[executable] = [2]string{kotkeymod, hotkey}

	osapi.Hotkeys = nil
	hotkeyConfig = make(map[string]string, len(activeHotkeys)+1)
	for exe, keys := range activeHotkeys {
		hotkeyConfig[exe] = keys[0] + "+" + keys[1]
	}
	config.WriteSettings(hotkeyConfig)

	osapi.StopKeylogger()
	osapi.AddHotkey(kotkeymod, hotkey, func() {
		TraceLog(fmt.Sprintf("Hotkey pressed: %s %s %s", executable, kotkeymod, hotkey))
	})

	osapi.StartKeylogger()
}
func (h *HotkeyService) ToggleHotkeys() {
	if osapi.LogKeys {
		osapi.StopKeylogger()
	} else {
		osapi.StartKeylogger()
	}
}
