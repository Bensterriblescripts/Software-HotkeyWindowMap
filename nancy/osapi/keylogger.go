package osapi

import (
	"runtime"
	"strconv"
	"strings"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

var LogKeys = true
var Hotkeys []Hotkey
var currentHotkeyID int32 = 0

type Hotkey struct {
	ID       int32
	Mod      string
	Key      string
	Callback func()
	Active   bool
}

var Keys = map[string]uintptr{
	"f1":  0x70,
	"f2":  0x71,
	"f3":  0x72,
	"f4":  0x73,
	"f5":  0x74,
	"f6":  0x75,
	"f7":  0x76,
	"f8":  0x77,
	"f9":  0x78,
	"f10": 0x79,
	"f11": 0x7A,
	"f12": 0x7B,
	"a":   0x41,
	"b":   0x42,
	"c":   0x43,
	"d":   0x44,
	"e":   0x45,
	"f":   0x46,
	"g":   0x47,
	"h":   0x48,
	"i":   0x49,
	"j":   0x4A,
	"k":   0x4B,
	"l":   0x4C,
	"m":   0x4D,
	"n":   0x4E,
	"o":   0x4F,
	"p":   0x50,
	"q":   0x51,
	"r":   0x52,
	"s":   0x53,
	"t":   0x54,
	"u":   0x55,
	"v":   0x56,
	"w":   0x57,
	"x":   0x58,
	"y":   0x59,
	"z":   0x5A,
	"0":   0x30,
	"1":   0x31,
	"2":   0x32,
	"3":   0x33,
	"4":   0x34,
	"5":   0x35,
	"6":   0x36,
	"7":   0x37,
	"8":   0x38,
	"9":   0x39,
}
var Modifiers = map[string]uintptr{
	"alt":     0x0001,
	"control": 0x0002,
	"shift":   0x0004,
	"win":     0x0008,
}

// Register all hotkeys in []Hotkeys and starts the message reciever loop
//
// Ensure all key strings are in lowercase
//
//	Hotkey{ID: 1, Mod: "alt", Key: "f1"},
//	Hotkey{ID: 2, Mod: "control", Key: "f2"},
//	Hotkey{ID: 3, Mod: "shift", Key: "f3"},
//	Hotkey{ID: 4, Mod: "win", Key: "f4"},
func StartKeylogger() {
	go func() {
		runtime.LockOSThread()
		defer runtime.UnlockOSThread()

		if len(Hotkeys) == 0 {
			ErrorLog("No hotkeys registered, add them to osapi.Hotkeys.\nE.g. osapi.Hotkeys = append(osapi.Hotkeys, osapi.Hotkey{ID: 1, Mod: \"alt\", Key: \"f1\"})")
			return
		}

		for _, hotkey := range Hotkeys {
			if _, ok := Keys[hotkey.Key]; !ok {
				ErrorLog("Invalid key: " + hotkey.Key)
				return
			}

			modifiers := uintptr(0)
			if strings.Contains(hotkey.Mod, "+") {
				modifiersSplit := strings.Split(hotkey.Mod, "+")
				for _, mod := range modifiersSplit {
					if _, ok := Modifiers[mod]; !ok {
						ErrorLog("Invalid modifier: " + mod)
						return
					} else {
						modifiers = uintptr(modifiers) | Modifiers[mod]
					}
				}
			} else {
				modifiers = Modifiers[hotkey.Mod]
			}

			if !registerHotKey(0, uintptr(hotkey.ID), modifiers, Keys[hotkey.Key]) {
				ErrorLog("Failed to register hotkey: " + hotkey.Mod + "+" + hotkey.Key)
				return
			}
			defer unregisterHotKey(0, uintptr(hotkey.ID))
			TraceLog("Registered hotkey: " + hotkey.Mod + " + " + hotkey.Key + " with ID: " + strconv.Itoa(int(hotkey.ID)))
		}

		LogKeys = true
		for LogKeys {
			var msg MSG
			r := GetMessage(&msg, 0, 0, 0)
			if r <= 0 {
				break
			}
			if msg.Message == 0x0312 {
				for _, hotkey := range Hotkeys {
					if msg.WParam == uintptr(hotkey.ID) {
						TraceLog("Hotkey Pressed: " + hotkey.Mod + " + " + hotkey.Key)
						hotkey.Callback()
					}
				}
			}
		}
	}()
}
func StopKeylogger() {
	LogKeys = false
	for _, hotkey := range Hotkeys {
		unregisterHotKey(0, uintptr(hotkey.ID))
	}
}
func AddHotkey(mod string, key string, callback func()) {
	if strings.Contains(mod, "+") {
		multimods := strings.Split(mod, "+")
		for _, mod := range multimods {
			if _, ok := Modifiers[mod]; !ok {
				ErrorLog("Invalid modifier: " + mod)
				return
			}
		}
	} else {
		if mod == "" {
			ErrorLog("Modifier is required")
			return
		}
		if _, ok := Modifiers[mod]; !ok {
			ErrorLog("Invalid modifier: " + mod)
			return
		}
	}

	if key == "" {
		ErrorLog("Key is required")
		return
	}
	if _, ok := Keys[key]; !ok {
		ErrorLog("Invalid key: " + key)
		return
	}

	if callback == nil {
		ErrorLog("Callback is required")
		return
	}

	hotkey := Hotkey{
		ID:       currentHotkeyID,
		Mod:      mod,
		Key:      key,
		Callback: callback,
		Active:   true,
	}
	Hotkeys = append(Hotkeys, hotkey)
	currentHotkeyID++

	TraceLog("Added hotkey: " + mod + " + " + key)
}

func registerHotKey(hwnd uintptr, id uintptr, modifiers uintptr, vk uintptr) bool {
	r, _, _ := procRegisterHotKey.Call(
		hwnd,
		id,
		modifiers,
		vk,
	)
	return r != 0
}
func unregisterHotKey(hwnd uintptr, id uintptr) bool {
	r, _, _ := procUnregisterHotKey.Call(
		hwnd,
		id,
	)
	return r != 0
}
