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

func (h *HotkeyService) GetAllActiveWindows() []osapi.Window {
	activeWindows = activeWindows[:0]
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
			switch window.WindowState {
			case "Borderless":
				TraceLog("Window is already borderless")
				return
			case "Borderless (Via Application)":
				TraceLog("Window borderless is managed by the application, skipping")
				return
			default:
				activeWindows[index].WindowState = "Borderless"
			}
			break
		}
	}
	osapi.SetBorderlessWindow(uintptr(handle))
}
func (h *HotkeyService) SetWindowed(handle int) {
	for index, window := range activeWindows {
		if window.Handle == uintptr(handle) {
			switch window.WindowState {
			case "Windowed":
				TraceLog("Window is already windowed")
				return
			case "Borderless (Via Application)":
				TraceLog("Window borderless is managed by the application, skipping")
				return
			default:
				activeWindows[index].WindowState = "Windowed"
			}
			break
		}
	}
	osapi.SetWindowWindowed(uintptr(handle))
}
func (h *HotkeyService) SetFocus(handle int) {
	osapi.SetFocus(uintptr(handle))
}

func (h *HotkeyService) GetAllHotkeys() map[string][2]string {
	if activeHotkeys == nil {
		activeHotkeys = make(map[string][2]string)
	}
	return activeHotkeys
}
func (h *HotkeyService) SetHotkey(executable string, hotkeymod string, hotkey string) {
	if activeHotkeys == nil {
		activeHotkeys = make(map[string][2]string)
	}

	if hotkeymod == "" || hotkey == "" {
		delete(activeHotkeys, executable)
	} else {
		if len(hotkey) > 1 {
			TraceLog("Invalid hotkey: key must be a single character")
			return
		}

		for exe, keys := range activeHotkeys {
			if exe != executable && keys[0] == hotkeymod && keys[1] == hotkey {
				TraceLog(fmt.Sprintf("Duplicate hotkey %s+%s: removing from %s", hotkeymod, hotkey, exe))
				delete(activeHotkeys, exe)
				break
			}
		}

		activeHotkeys[executable] = [2]string{hotkeymod, hotkey}
	}

	osapi.Hotkeys = nil
	hotkeyConfig := make(map[string]string, len(activeHotkeys))
	for exe, keys := range activeHotkeys {
		hotkeyConfig[exe] = keys[0] + "+" + keys[1]
	}
	config.WriteSettings(hotkeyConfig)

	osapi.StopKeylogger()
	if len(activeHotkeys) == 0 {
		return
	}
	for exe, keys := range activeHotkeys {
		exe, keys := exe, keys
		osapi.AddHotkey(keys[0], keys[1], func() {
			TraceLog(fmt.Sprintf("Hotkey pressed: %s %s %s", exe, keys[0], keys[1]))
		})
	}

	osapi.StartKeylogger()
}
func (h *HotkeyService) ToggleHotkeys() {
	if osapi.LogKeys {
		osapi.StopKeylogger()
	} else {
		osapi.StartKeylogger()
	}
}
