//pub mod cmd::alias_list;
// use crate::cmd::{self, configx};
use clap::command;

use super::{
    admin, alias, cp, find, lsmain, mb,
    put::{self},
    rb::{self},
    rm,
};

// static APPNAME: &str = "rustfs-cli [FLAGS] COMMAND [COMMAND FLAGS | -h] [ARGUMENTS...]";
static ABOUT: &str = "manager client for rustfs and minio";
use clap::Parser;

#[derive(Parser)]
#[command(name = "rustfs-cli")]
#[command(about = ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// 定义主命令，包含 config 和 run 子命令
#[derive(clap::Subcommand)]
enum Commands {
    #[command(about = "manage server credentials in configuration file")]
    Alias {
        #[command(subcommand)]
        subcommand: alias::AliasCommands, // 从 config 模块导入
    },
    #[command(about = "manage MinIO servers")]
    Admin {
        #[command(subcommand)]
        subcommand: admin::AdminCommands, // 从 run 模块导入
    },
    #[command(about = "list buckets and objects")]
    Ls(lsmain::LsOptions),
    #[command(about = "make a bucket")]
    Mb(mb::MbOptions),
    #[command(about = "remove a bucket")]
    Rb(rb::RbOptions),
    #[command(about = "upload an object to a bucket")]
    Put(put::PutOptions),
    #[command(about = "remove object(s)")]
    Rm(rm::RmOptions),
    #[command(about = "copy objects")]
    Cp(cp::CpOptions),
    #[command(about = "search for objects")]
    Find(find::FindOptions),
}

async fn register_app() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Alias { subcommand } => {
            //handle_config_commands(subcommand);
            alias::handle_alias_commands(subcommand).await;
        }
        Commands::Admin { subcommand } => {
            admin::handle_admin_commands(subcommand).await;
        }
        Commands::Ls(opt) => {
            println!("xxxxxxxxxxx");
            lsmain::handle_ls_commands(opt).await;
        }
        Commands::Mb(opt) => {
            mb::handle_mb_command(opt).await;
        }
        Commands::Rb(opt) => {
            rb::handle_rb_command(opt).await;
        }
        Commands::Put(opt) => {
            match put::handle_put_command(opt).await {
                Ok(_) => {
                    // Handle the success case here if needed
                    // For example, you might want to log a success message
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e); // Log the error or take other actions
                }
            }
        }
        Commands::Rm(opt) => {
            match rm::handle_rm_command(opt).await {
                Ok(_) => {
                    println!("remove successed");
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e); // Log the error or take other actions
                }
            }
        }
        Commands::Cp(opt) => {
            match cp::handle_cp_command(opt).await {
                Ok(_) => {
                    println!("remove successed");
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e); // Log the error or take other actions
                }
            }
        }
        Commands::Find(opt) => {
            find::handle_find_command(opt).await;
        }
    }
}

// fn register_app2() {
//     let mut app = Command::new(APPNAME)
//         .version("1.0")
//         .author("Your Name <you@example.com>")
//         .about(ABOUT)

//         // alias 子命令及其子命令
//         .subcommand(
//             Command::new("alias")
//                 .about("Manage server credentials")
//                 .subcommand(
//                     Command::new("set")
//                         .about("Set a new alias to configuration file"),
//                 )
//                 .subcommand(
//                     Command::new("list")
//                         .about("List aliases in configuration file")
//                         .arg(Arg::new("FILE").help("The file to list").required(true)),
//                 )
//                 .subcommand(
//                     Command::new("remove")
//                         .about("Remove an alias from configuration file")
//                         .arg(Arg::new("FILE").help("The file to remove").required(true)),
//                 )
//                 .subcommand(
//                     Command::new("import")
//                         .about("Import configuration info to configuration file from a JSON formatted string")
//                         .arg(Arg::new("FILE").help("The file to import").required(true)),
//                 )
//                 .subcommand(
//                     Command::new("export")
//                         .about("Export configuration info to stdout")
//                         .arg(Arg::new("FILE").help("The file to export").required(true)),
//                 ),
//         )

//         // admin 子命令及其子命令
//         .subcommand(
//             Command::new("admin")
//                 .about("Manage the system")
//                 .subcommand(
//                     Command::new("status")
//                         .about("Show system status"),
//                 )
//                 .subcommand(
//                     Command::new("restart")
//                         .about("Restart the system"),
//                 )
//                 .subcommand(
//                     Command::new("stop")
//                         .about("Stop the system"),
//                 ),
//         );

//     let matches = app.clone().get_matches(); // 先获取匹配结果
//     //let mut app2 = app.clone();

//     match matches.subcommand() {
//         Some(("alias", sub_matches)) => {
//             // 处理 alias 的子命令
//             match sub_matches.subcommand() {
//                 Some(("list", _sub_sub_matches)) => {
//                     // let file = sub_sub_matches.get_one::<String>("FILE").unwrap();
//                     // println!("Listing alias for file: {}", file);

//                     cmd::cmd::alias_list();
//                 }
//                 Some(("remove", sub_sub_matches)) => {
//                     let file = sub_sub_matches.get_one::<String>("FILE").unwrap();
//                     println!("Removing alias for file: {}", file);
//                 }
//                 Some(("set", _)) => {
//                     println!("Setting a new alias...");
//                 }
//                 _ => {
//                     // 没有匹配到有效的 alias 子命令时，显示帮助信息
//                     app.find_subcommand_mut("alias").unwrap().print_help().unwrap();
//                     println!();
//                     return; // 打印帮助后退出
//                 }
//             }
//         }
//         Some(("admin", sub_matches)) => {
//             // 处理 admin 的子命令
//             match sub_matches.subcommand() {
//                 Some(("status", _)) => {
//                     println!("Showing system status...");
//                 }
//                 Some(("restart", _)) => {
//                     println!("Restarting the system...");
//                 }
//                 Some(("stop", _)) => {
//                     println!("Stopping the system...");
//                 }
//                 _ => {
//                     // 没有匹配到有效的 admin 子命令时，显示帮助信息
//                     app.find_subcommand_mut("admin").unwrap().print_help().unwrap();
//                     println!();
//                     return;
//                 }
//             }
//         }
//         _ => {
//             // 当没有提供任何子命令时，打印主命令帮助信息
//             app.print_help().unwrap();
//             println!();
//         }
//     }
// }
// func registerApp(name string) *cli.App {
// 	cli.HelpFlag = cli.BoolFlag{
// 		Name:  "help, h",
// 		Usage: "show help",
// 	}

// 	// Override default cli version printer
// 	cli.VersionPrinter = printMCVersion

// 	app := cli.NewApp()
// 	app.Name = name
// 	app.Action = func(ctx *cli.Context) error {
// 		if strings.HasPrefix(ReleaseTag, "RELEASE.") {
// 			// Check for new updates from dl.min.io.
// 			checkUpdate(ctx)
// 		}

// 		if ctx.Bool("autocompletion") || ctx.GlobalBool("autocompletion") {
// 			// Install shell completions
// 			installAutoCompletion()
// 			return nil
// 		}

// 		if ctx.Args().First() == "" {
// 			showAppHelpAndExit(ctx)
// 		}

// 		commandNotFound(ctx, app.Commands)
// 		return exitStatus(globalErrorExitStatus)
// 	}

// 	app.Before = registerBefore
// 	app.HideHelpCommand = true
// 	app.Usage = "MinIO Client for object storage and filesystems."
// 	app.Commands = appCmds
// 	app.Author = "MinIO, Inc."
// 	app.Version = ReleaseTag
// 	app.Flags = append(mcFlags, globalFlags...)
// 	app.CustomAppHelpTemplate = mcHelpTemplate
// 	app.EnableBashCompletion = true
// 	app.OnUsageError = onUsageError
// 	app.After = func(*cli.Context) error {
// 		globalExpiringCerts.Range(func(k, v interface{}) bool {
// 			host := k.(string)
// 			expires := v.(time.Time)
// 			fmt.Fprintf(os.Stderr, "\n")
// 			fmt.Fprintf(os.Stderr, "== WARN: `%s` certificate will expire in %s. Renew soon to avoid outage.\n", host, expires)
// 			fmt.Fprintf(os.Stderr, "\n")
// 			return true
// 		})
// 		return nil
// 	}

// 	if isTerminal() && !globalPagerDisabled {
// 		app.HelpWriter = globalHelpPager
// 	} else {
// 		app.HelpWriter = os.Stdout
// 	}

// 	return app
// }

pub async fn main(_args: Vec<String>) {
    register_app().await;
}

// Copyright (c) 2015-2022 MinIO, Inc.
//
// This file is part of MinIO Object Storage stack
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// package cmd

// import (
// 	"bytes"
// 	"errors"
// 	"fmt"
// 	"os"
// 	"os/exec"
// 	"path/filepath"
// 	"reflect"
// 	"regexp"
// 	"runtime"
// 	"sort"
// 	"strconv"
// 	"strings"
// 	"syscall"
// 	"time"

// 	"github.com/inconshreveable/mousetrap"
// 	"github.com/minio/cli"
// 	"github.com/minio/mc/pkg/probe"
// 	"github.com/minio/minio-go/v7/pkg/set"
// 	"github.com/minio/pkg/v3/console"
// 	"github.com/minio/pkg/v3/env"
// 	"github.com/minio/pkg/v3/trie"
// 	"github.com/minio/pkg/v3/words"
// 	"golang.org/x/term"

// 	completeinstall "github.com/posener/complete/cmd/install"
// )

// // global flags for mc.
// var mcFlags = []cli.Flag{
// 	cli.BoolFlag{
// 		Name:  "autocompletion",
// 		Usage: "install auto-completion for your shell",
// 	},
// }

// // Help template for mc
// var mcHelpTemplate = `NAME:
//   {{.Name}} - {{.Usage}}

// USAGE:
//   {{.Name}} {{if .VisibleFlags}}[FLAGS] {{end}}COMMAND{{if .VisibleFlags}} [COMMAND FLAGS | -h]{{end}} [ARGUMENTS...]

// COMMANDS:
//   {{range .VisibleCommands}}{{join .Names ", "}}{{ "\t" }}{{.Usage}}
//   {{end}}{{if .VisibleFlags}}
// GLOBAL FLAGS:
//   {{range .VisibleFlags}}{{.}}
//   {{end}}{{end}}
// TIP:
//   Use '{{.Name}} --autocompletion' to enable shell autocompletion

// COPYRIGHT:
//   Copyright (c) 2015-` + CopyrightYear + ` MinIO, Inc.

// LICENSE:
//   GNU AGPLv3 <https://www.gnu.org/licenses/agpl-3.0.html>
// `

// func init() {
// 	if env.IsSet(mcEnvConfigFile) {
// 		configFile := env.Get(mcEnvConfigFile, "")
// 		fatalIf(readAliasesFromFile(configFile).Trace(configFile), "Unable to parse "+configFile)
// 	}
// 	if runtime.GOOS == "windows" {
// 		if mousetrap.StartedByExplorer() {
// 			fmt.Printf("Don't double-click %s\n", os.Args[0])
// 			fmt.Println("You need to open cmd.exe/PowerShell and run it from the command line")
// 			fmt.Println("Press the Enter Key to Exit")
// 			fmt.Scanln()
// 			os.Exit(1)
// 		}
// 	}
// }

// // Main starts mc application
// func Main(args []string) error {
// 	if len(args) > 1 {
// 		switch args[1] {
// 		case "mc", filepath.Base(args[0]):
// 			mainComplete()
// 			return nil
// 		}
// 	}

// 	// ``MC_PROFILER`` supported options are [cpu, mem, block, goroutine].
// 	if p := os.Getenv("MC_PROFILER"); p != "" {
// 		profilers := strings.Split(p, ",")
// 		if e := enableProfilers(mustGetProfileDir(), profilers); e != nil {
// 			console.Fatal(e)
// 		}
// 	}

// 	probe.Init() // Set project's root source path.
// 	probe.SetAppInfo("Release-Tag", ReleaseTag)
// 	probe.SetAppInfo("Commit", ShortCommitID)

// 	// Fetch terminal size, if not available, automatically
// 	// set globalQuiet to true on non-window.
// 	if w, h, e := term.GetSize(int(os.Stdout.Fd())); e != nil {
// 		globalQuiet = runtime.GOOS != "windows"
// 	} else {
// 		globalTermWidth, globalTermHeight = w, h
// 	}

// 	// Set the mc app name.
// 	appName := filepath.Base(args[0])
// 	if runtime.GOOS == "windows" && strings.HasSuffix(strings.ToLower(appName), ".exe") {
// 		// Trim ".exe" from Windows executable.
// 		appName = appName[:strings.LastIndex(appName, ".")]
// 	}

// 	// Monitor OS exit signals and cancel the global context in such case
// 	go trapSignals(os.Interrupt, syscall.SIGTERM, syscall.SIGKILL)

// 	globalHelpPager = newTermPager()
// 	// Wait until the user quits the pager
// 	defer globalHelpPager.WaitForExit()

// 	parsePagerDisableFlag(args)
// 	// Run the app
// 	return registerApp(appName).Run(args)
// }

// func flagValue(f cli.Flag) reflect.Value {
// 	fv := reflect.ValueOf(f)
// 	for fv.Kind() == reflect.Ptr {
// 		fv = reflect.Indirect(fv)
// 	}
// 	return fv
// }

// func visibleFlags(fl []cli.Flag) []cli.Flag {
// 	visible := []cli.Flag{}
// 	for _, flag := range fl {
// 		field := flagValue(flag).FieldByName("Hidden")
// 		if !field.IsValid() || !field.Bool() {
// 			visible = append(visible, flag)
// 		}
// 	}
// 	return visible
// }

// // Function invoked when invalid flag is passed
// func onUsageError(ctx *cli.Context, err error, _ bool) error {
// 	type subCommandHelp struct {
// 		flagName string
// 		usage    string
// 	}

// 	// Calculate the maximum width of the flag name field
// 	// for a good looking printing
// 	vflags := visibleFlags(ctx.Command.Flags)
// 	help := make([]subCommandHelp, len(vflags))
// 	maxWidth := 0
// 	for i, f := range vflags {
// 		s := strings.Split(f.String(), "\t")
// 		if len(s[0]) > maxWidth {
// 			maxWidth = len(s[0])
// 		}

// 		help[i] = subCommandHelp{flagName: s[0], usage: s[1]}
// 	}
// 	maxWidth += 2

// 	var errMsg strings.Builder

// 	// Do the good-looking printing now
// 	fmt.Fprintln(&errMsg, "Invalid command usage,", err.Error())
// 	if len(help) > 0 {
// 		fmt.Fprintln(&errMsg, "\nSUPPORTED FLAGS:")
// 		for _, h := range help {
// 			spaces := string(bytes.Repeat([]byte{' '}, maxWidth-len(h.flagName)))
// 			fmt.Fprintf(&errMsg, "   %s%s%s\n", h.flagName, spaces, h.usage)
// 		}
// 	}
// 	console.Fatal(errMsg.String())
// 	return err
// }

// // Function invoked when invalid command is passed.
// func commandNotFound(ctx *cli.Context, cmds []cli.Command) {
// 	command := ctx.Args().First()
// 	if command == "" {
// 		cli.ShowCommandHelp(ctx, command)
// 		return
// 	}
// 	msg := fmt.Sprintf("`%s` is not a recognized command. Get help using `--help` flag.", command)
// 	commandsTree := trie.NewTrie()
// 	for _, cmd := range cmds {
// 		commandsTree.Insert(cmd.Name)
// 	}
// 	closestCommands := findClosestCommands(commandsTree, command)
// 	if len(closestCommands) > 0 {
// 		msg += "\n\nDid you mean one of these?\n"
// 		if len(closestCommands) == 1 {
// 			cmd := closestCommands[0]
// 			msg += fmt.Sprintf("        `%s`", cmd)
// 		} else {
// 			for _, cmd := range closestCommands {
// 				msg += fmt.Sprintf("        `%s`\n", cmd)
// 			}
// 		}
// 	}
// 	fatalIf(errDummy().Trace(), msg)
// }

// // Check for sane config environment early on and gracefully report.
// func checkConfig() {
// 	// Refresh the config once.
// 	loadMcConfig = loadMcConfigFactory()
// 	// Ensures config file is sane.
// 	config, err := loadMcConfig()
// 	// Verify if the path is accesible before validating the config
// 	fatalIf(err.Trace(mustGetMcConfigPath()), "Unable to access configuration file.")

// 	// Validate and print error messges
// 	ok, errMsgs := validateConfigFile(config)
// 	if !ok {
// 		var errorMsg bytes.Buffer
// 		for index, errMsg := range errMsgs {
// 			// Print atmost 10 errors
// 			if index > 10 {
// 				break
// 			}
// 			errorMsg.WriteString(errMsg + "\n")
// 		}
// 		console.Fatal(errorMsg.String())
// 	}
// }

// func migrate() {
// 	// Fix broken config files if any.
// 	fixConfig()

// 	// Migrate config files if any.
// 	migrateConfig()

// 	// Migrate shared urls if any.
// 	migrateShare()
// }

// // initMC - initialize 'mc'.
// func initMC() {
// 	// Check if mc config exists.
// 	if !isMcConfigExists() {
// 		err := saveMcConfig(newMcConfig())
// 		fatalIf(err.Trace(), "Unable to save new mc config.")

// 		if !globalQuiet && !globalJSON {
// 			console.Infoln("Configuration written to `" + mustGetMcConfigPath() + "`. Please update your access credentials.")
// 		}
// 	}

// 	// Check if mc share directory exists.
// 	if !isShareDirExists() {
// 		initShareConfig()
// 	}

// 	// Check if certs dir exists
// 	if !isCertsDirExists() {
// 		fatalIf(createCertsDir().Trace(), "Unable to create `CAs` directory.")
// 	}

// 	// Check if CAs dir exists
// 	if !isCAsDirExists() {
// 		fatalIf(createCAsDir().Trace(), "Unable to create `CAs` directory.")
// 	}

// 	// Load all authority certificates present in CAs dir
// 	loadRootCAs()
// }

// func getShellName() (string, bool) {
// 	shellName := os.Getenv("SHELL")
// 	if shellName != "" || runtime.GOOS == "windows" {
// 		return strings.ToLower(filepath.Base(shellName)), true
// 	}

// 	ppid := os.Getppid()
// 	cmd := exec.Command("ps", "-p", strconv.Itoa(ppid), "-o", "comm=")
// 	ppName, err := cmd.Output()
// 	if err != nil {
// 		fatalIf(probe.NewError(err), "Failed to enable autocompletion. Cannot determine shell type and "+
// 			"no SHELL environment variable found")
// 	}
// 	shellName = strings.TrimSpace(string(ppName))
// 	return strings.ToLower(filepath.Base(shellName)), false
// }

// func installAutoCompletion() {
// 	if runtime.GOOS == "windows" {
// 		console.Infoln("autocompletion feature is not available for this operating system")
// 		return
// 	}

// 	shellName, ok := getShellName()
// 	if !ok {
// 		console.Infoln("No 'SHELL' env var. Your shell is auto determined as '" + shellName + "'.")
// 	} else {
// 		console.Infoln("Your shell is set to '" + shellName + "', by env var 'SHELL'.")
// 	}

// 	supportedShellsSet := set.CreateStringSet("bash", "zsh", "fish")
// 	if !supportedShellsSet.Contains(shellName) {
// 		fatalIf(probe.NewError(errors.New("")),
// 			"'"+shellName+"' is not a supported shell. "+
// 				"Supported shells are: bash, zsh, fish")
// 	}

// 	e := completeinstall.Install(filepath.Base(os.Args[0]))
// 	var printMsg string
// 	if e != nil && strings.Contains(e.Error(), "* already installed") {
// 		errStr := e.Error()[strings.Index(e.Error(), "\n")+1:]
// 		re := regexp.MustCompile(`[::space::]*\*.*` + shellName + `.*`)
// 		relatedMsg := re.FindStringSubmatch(errStr)
// 		if len(relatedMsg) > 0 {
// 			printMsg = "\n" + relatedMsg[0]
// 		} else {
// 			printMsg = ""
// 		}
// 	}
// 	if printMsg != "" {
// 		if completeinstall.IsInstalled(filepath.Base(os.Args[0])) || completeinstall.IsInstalled("mc") {
// 			console.Infoln("autocompletion is enabled.", printMsg)
// 		} else {
// 			fatalIf(probe.NewError(e), "Unable to install auto-completion.")
// 		}
// 	} else {
// 		console.Infoln("enabled autocompletion in your '" + shellName + "' rc file. Please restart your shell.")
// 	}
// }

// func registerBefore(ctx *cli.Context) error {
// 	deprecatedFlagsWarning(ctx)

// 	if ctx.IsSet("config-dir") {
// 		// Set the config directory.
// 		setMcConfigDir(ctx.String("config-dir"))
// 	} else if ctx.GlobalIsSet("config-dir") {
// 		// Set the config directory.
// 		setMcConfigDir(ctx.GlobalString("config-dir"))
// 	}

// 	// Set global flags.
// 	setGlobalsFromContext(ctx)

// 	// Migrate any old version of config / state files to newer format.
// 	migrate()

// 	// Initialize default config files.
// 	initMC()

// 	// Check if config can be read.
// 	checkConfig()

// 	return nil
// }

// // findClosestCommands to match a given string with commands trie tree.
// func findClosestCommands(commandsTree *trie.Trie, command string) []string {
// 	closestCommands := commandsTree.PrefixMatch(command)
// 	sort.Strings(closestCommands)
// 	// Suggest other close commands - allow missed, wrongly added and even transposed characters
// 	for _, value := range commandsTree.Walk(commandsTree.Root()) {
// 		if sort.SearchStrings(closestCommands, value) < len(closestCommands) {
// 			continue
// 		}
// 		// 2 is arbitrary and represents the max allowed number of typed errors
// 		if words.DamerauLevenshteinDistance(command, value) < 2 {
// 			closestCommands = append(closestCommands, value)
// 		}
// 	}
// 	return closestCommands
// }

// // Check for updates and print a notification message
// func checkUpdate(ctx *cli.Context) {
// 	// Do not print update messages, if quiet flag is set.
// 	if ctx.Bool("quiet") || ctx.GlobalBool("quiet") {
// 		// Its OK to ignore any errors during doUpdate() here.
// 		if updateMsg, _, currentReleaseTime, latestReleaseTime, _, err := getUpdateInfo("", 2*time.Second); err == nil {
// 			printMsg(updateMessage{
// 				Status:  "success",
// 				Message: updateMsg,
// 			})
// 		} else {
// 			printMsg(updateMessage{
// 				Status:  "success",
// 				Message: prepareUpdateMessage("Run `mc update`", latestReleaseTime.Sub(currentReleaseTime)),
// 			})
// 		}
// 	}
// }

// var appCmds = []cli.Command{
// 	aliasCmd,
// 	adminCmd,
// 	anonymousCmd,
// 	batchCmd,
// 	cpCmd,
// 	catCmd,
// 	configCmd,
// 	corsCmd,
// 	diffCmd,
// 	duCmd,
// 	encryptCmd,
// 	eventCmd,
// 	findCmd,
// 	getCmd,
// 	headCmd,
// 	ilmCmd,
// 	idpCmd,
// 	licenseCmd,
// 	legalHoldCmd,
// 	lsCmd,
// 	mbCmd,
// 	mvCmd,
// 	mirrorCmd,
// 	odCmd,
// 	pingCmd,
// 	policyCmd,
// 	pipeCmd,
// 	putCmd,
// 	quotaCmd,
// 	rmCmd,
// 	retentionCmd,
// 	rbCmd,
// 	replicateCmd,
// 	readyCmd,
// 	sqlCmd,
// 	statCmd,
// 	supportCmd,
// 	shareCmd,
// 	treeCmd,
// 	tagCmd,
// 	undoCmd,
// 	updateCmd,
// 	versionCmd,
// 	watchCmd,
// }

// func printMCVersion(c *cli.Context) {
// 	fmt.Fprintf(c.App.Writer, "%s version %s (commit-id=%s)\n", c.App.Name, c.App.Version, CommitID)
// 	fmt.Fprintf(c.App.Writer, "Runtime: %s %s/%s\n", runtime.Version(), runtime.GOOS, runtime.GOARCH)
// 	fmt.Fprintf(c.App.Writer, "Copyright (c) 2015-%s MinIO, Inc.\n", CopyrightYear)
// 	fmt.Fprintf(c.App.Writer, "License GNU AGPLv3 <https://www.gnu.org/licenses/agpl-3.0.html>\n")
// }

// func registerApp(name string) *cli.App {
// 	cli.HelpFlag = cli.BoolFlag{
// 		Name:  "help, h",
// 		Usage: "show help",
// 	}

// 	// Override default cli version printer
// 	cli.VersionPrinter = printMCVersion

// 	app := cli.NewApp()
// 	app.Name = name
// 	app.Action = func(ctx *cli.Context) error {
// 		if strings.HasPrefix(ReleaseTag, "RELEASE.") {
// 			// Check for new updates from dl.min.io.
// 			checkUpdate(ctx)
// 		}

// 		if ctx.Bool("autocompletion") || ctx.GlobalBool("autocompletion") {
// 			// Install shell completions
// 			installAutoCompletion()
// 			return nil
// 		}

// 		if ctx.Args().First() == "" {
// 			showAppHelpAndExit(ctx)
// 		}

// 		commandNotFound(ctx, app.Commands)
// 		return exitStatus(globalErrorExitStatus)
// 	}

// 	app.Before = registerBefore
// 	app.HideHelpCommand = true
// 	app.Usage = "MinIO Client for object storage and filesystems."
// 	app.Commands = appCmds
// 	app.Author = "MinIO, Inc."
// 	app.Version = ReleaseTag
// 	app.Flags = append(mcFlags, globalFlags...)
// 	app.CustomAppHelpTemplate = mcHelpTemplate
// 	app.EnableBashCompletion = true
// 	app.OnUsageError = onUsageError
// 	app.After = func(*cli.Context) error {
// 		globalExpiringCerts.Range(func(k, v interface{}) bool {
// 			host := k.(string)
// 			expires := v.(time.Time)
// 			fmt.Fprintf(os.Stderr, "\n")
// 			fmt.Fprintf(os.Stderr, "== WARN: `%s` certificate will expire in %s. Renew soon to avoid outage.\n", host, expires)
// 			fmt.Fprintf(os.Stderr, "\n")
// 			return true
// 		})
// 		return nil
// 	}

// 	if isTerminal() && !globalPagerDisabled {
// 		app.HelpWriter = globalHelpPager
// 	} else {
// 		app.HelpWriter = os.Stdout
// 	}

// 	return app
// }

// // mustGetProfilePath must get location that the profile will be written to.
// func mustGetProfileDir() string {
// 	return filepath.Join(mustGetMcConfigDir(), globalProfileDir)
// }

// func showCommandHelpAndExit(cliCtx *cli.Context, code int) {
// 	cli.ShowCommandHelp(cliCtx, cliCtx.Command.Name)
// 	// Wait until the user quits the pager
// 	globalHelpPager.WaitForExit()
// 	os.Exit(code)
// }

// func showAppHelpAndExit(cliCtx *cli.Context) {
// 	cli.ShowAppHelp(cliCtx)
// 	// Wait until the user quits the pager
// 	globalHelpPager.WaitForExit()
// 	os.Exit(globalErrorExitStatus)
// }
