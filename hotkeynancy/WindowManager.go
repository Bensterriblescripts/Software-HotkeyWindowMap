package main

import (
	"github.com/Bensterriblescripts/Lib-Handlers/osapi"
)

type WindowManager struct{}

var activeWindows []osapi.Window
var activeHotkeys map[int]string
var ignoredWindows map[string]bool

func (h *WindowManager) GetAllActiveWindows() []osapi.Window {
	activeWindows = []osapi.Window{}
	allWindows := osapi.GetAllActiveWindows()
	for _, window := range allWindows {
		if ignoredWindows[window.Title] == true {
			continue
		} else {
			activeWindows = append(activeWindows, window)
		}
	}
	return activeWindows
}
func (h *WindowManager) AddIgnoredWindow(title string) {
	ignoredWindows[title] = true
}
func (h *WindowManager) RemoveIgnoredWindow(title string) {
	ignoredWindows[title] = false
}
func (h *WindowManager) SetBorderlessFullscreen(handle int) {
	osapi.SetBorderlessWindow(uintptr(handle))
}
func (h *WindowManager) SetWindowed(handle int) {
	osapi.SetWindowWindowed(uintptr(handle))
}
func (h *WindowManager) GetAllHotkeys() map[int]string {
	return activeHotkeys
}
func (h *WindowManager) SetHotkey(handle int, hotkey string) {
}
func (h *WindowManager) IgnoreWindow(title string) []string {
	return []string{"Ctrl", "Alt", "Shift", "Ctrl+Alt", "Ctrl+Shift", "Alt+Shift"}
}
