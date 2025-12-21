package main

import (
	"embed"
	_ "embed"
	"log"
	"nancy/osapi"

	"github.com/Bensterriblescripts/Lib-Handlers/config"
	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
	"github.com/wailsapp/wails/v3/pkg/application"
)

//go:embed all:frontend/dist
var assets embed.FS

func main() {
	AppName = "HotkeyNancy"
	TraceDebug = true
	ConsoleLogging = true
	InitLogs()
	config.ReadConfig()

	osapi.Hotkeys = append(osapi.Hotkeys, osapi.Hotkey{Mod: "alt", Key: "f1"})
	osapi.Hotkeys = append(osapi.Hotkeys, osapi.Hotkey{Mod: "alt", Key: "f2"})
	go osapi.StartKeylogger()

	app := application.New(application.Options{
		Name:        "HotkeyNancy",
		Description: "Hotkey and borderless window manager",
		Services: []application.Service{
			application.NewService(&WindowManager{}),
		},
		Assets: application.AssetOptions{
			Handler: application.AssetFileServerFS(assets),
		},
	})
	app.Window.NewWithOptions(application.WebviewWindowOptions{
		Title:            "Hotkey Nancy",
		BackgroundColour: application.NewRGB(27, 38, 54),
		Width:            1200,
		Height:           800,
		URL:              "/",
	})

	err := app.Run()
	if err != nil {
		log.Fatal(err)
	}
}
