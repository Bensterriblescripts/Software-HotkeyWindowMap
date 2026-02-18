package main

import (
	"os"
	"path/filepath"
	"strings"
	"time"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

type LogsService struct {
	Name string
	Date string
}

type LogFile struct {
	ApplicationName string
	Date            time.Time
	Level           string
	Logs            []string
}

const logsRoot = "C:\\Local\\Logs"

func (l *LogsService) GetLogs() []LogFile {
	var logs []LogFile

	if appDirs, errExists := ErrorExists(os.ReadDir(logsRoot)); errExists {
		ErrorLog("Failed to read logs root directory " + logsRoot)
	} else {
		for _, appDir := range appDirs {
			if !appDir.IsDir() {
				continue
			}
			appName := appDir.Name()
			appPath := filepath.Join(logsRoot, appName)

			if files, errExists := ErrorExists(os.ReadDir(appPath)); errExists {
				ErrorLog("Failed to read application log directory " + appPath)
			} else {
				for _, file := range files {
					if file.IsDir() {
						continue
					}

					name := file.Name()
					if level, date, errExists := parseLogFilename(name); errExists {
						ErrorLog("Failed to parse log filename " + name + " in " + appPath)
					} else {
						logs = append(logs, LogFile{
							ApplicationName: appName,
							Date:            date,
							Level:           level,
						})
					}
				}
			}
		}
	}

	return logs
}

func parseLogFilename(name string) (string, time.Time, bool) {
	name = strings.TrimSuffix(name, filepath.Ext(name))
	parts := strings.SplitN(name, "-", 2)
	if len(parts) != 2 {
		return "", time.Time{}, true
	}
	if date, errExists := ErrorExists(time.Parse("2006-1-2", parts[1])); errExists {
		return "", time.Time{}, true
	} else {
		return parts[0], date, false
	}
}
