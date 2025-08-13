package cli

import (
	"context"
	"fmt"
	"mtu-optimizer/domain/entities"
	"mtu-optimizer/domain/ports"
	"mtu-optimizer/presentation/cli/output"
	"mtu-optimizer/presentation/cli/progress"
	"time"

	"github.com/spf13/cobra"
)

type Config struct {
	Target               string
	MinMTU               int
	MaxMTU               int
	Step                 int
	Requests             int
	TimeoutMs            int
	Interface            string
	Format               string
	Verbose              bool
	Debug                bool
	SkipConnectivityTest bool
	MaxConcurrent        int
}

func NewRootCommand(optimizer ports.MTUOptimizer) *cobra.Command {
	config := &Config{}

	cmd := &cobra.Command{
		Use:   "mtu",
		Short: "MTU Optimizer - Find optimal MTU for your network connection",
		Long: `MTU Optimizer finds the optimal MTU size by testing different values
and analyzing latency and packet loss.`,
		Version: "1.0.0",
		RunE: func(cmd *cobra.Command, args []string) error {
			return runOptimization(optimizer, config)
		},
	}

	cmd.Flags().StringVar(&config.Target, "target", "8.8.8.8", "Target IP address for ping")
	cmd.Flags().IntVar(&config.MinMTU, "min-mtu", 1200, "Minimum MTU size to test")
	cmd.Flags().IntVar(&config.MaxMTU, "max-mtu", 1500, "Maximum MTU size to test")
	cmd.Flags().IntVar(&config.Step, "step", 8, "Increment between MTU tests")
	cmd.Flags().IntVar(&config.Requests, "requests", 5, "Number of pings per MTU size")
	cmd.Flags().IntVar(&config.TimeoutMs, "timeout-ms", 3000, "Timeout per ping in milliseconds")
	cmd.Flags().StringVar(&config.Interface, "interface", "auto", "Network interface to use")
	cmd.Flags().StringVar(&config.Format, "format", "text", "Output format (text, json)")
	cmd.Flags().BoolVar(&config.Verbose, "verbose", false, "Enable verbose output")
	cmd.Flags().BoolVar(&config.Debug, "debug", false, "Enable debug output")
	cmd.Flags().BoolVar(&config.SkipConnectivityTest, "skip-connectivity-test", false, "Skip initial connectivity test")
	cmd.Flags().IntVar(&config.MaxConcurrent, "max-concurrent", 10, "Maximum concurrent operations")

	return cmd
}

func runOptimization(optimizer ports.MTUOptimizer, config *Config) error {
	if config.Debug {
		fmt.Printf("Starting MTU optimization with config: %+v\n", config)
	}

	if !config.SkipConnectivityTest {
		if config.Verbose {
			fmt.Printf("Testing connectivity to %s...\n", config.Target)
		}
	}

	if config.Verbose {
		fmt.Printf("Testing MTU range %d-%d with step %d\n", config.MinMTU, config.MaxMTU, config.Step)
		fmt.Printf("Sending %d requests per MTU, timeout %dms\n", config.Requests, config.TimeoutMs)
		fmt.Println()
	}

	ctx := context.Background()
	mtuRange := entities.MTURange{
		Min:  config.MinMTU,
		Max:  config.MaxMTU,
		Step: config.Step,
	}

	timeout := time.Duration(config.TimeoutMs) * time.Millisecond

	var result *entities.OptimizationResult
	var err error

	if config.Format == "json" {
		result, err = optimizer.OptimizeRange(ctx, config.Target, mtuRange, config.Requests, timeout)
	} else {
		totalMTUs := (config.MaxMTU-config.MinMTU)/config.Step + 1
		progressBar := progress.NewProgressBar(totalMTUs)

		fmt.Printf("Testing MTU range %d-%d (step %d) against %s\n",
			config.MinMTU, config.MaxMTU, config.Step, config.Target)

		progressCallback := func(current, total int, message string) {
			progressBar.Update(current, message)
		}

		result, err = optimizer.OptimizeRangeWithProgress(ctx, config.Target, mtuRange, config.Requests, timeout, progressCallback)
		progressBar.Finish()
		fmt.Println()
	}

	if err != nil {
		return fmt.Errorf("MTU optimization failed: %w", err)
	}

	outputConfig := &output.Config{
		Format:  config.Format,
		Verbose: config.Verbose,
		ShowAll: true,
	}

	return output.DisplayOptimizationResult(result, outputConfig)
}
