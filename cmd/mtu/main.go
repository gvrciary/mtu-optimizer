package main

import (
	"fmt"
	"mtu-optimizer/infrastructure/ping"
	"mtu-optimizer/presentation/cli"
	"mtu-optimizer/usecases"
	"os"
)

func main() {
	pingService := ping.NewICMPPingService()
	optimizer := usecases.NewMTUOptimizerUseCase(pingService)

	rootCmd := cli.NewRootCommand(optimizer)

	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}
