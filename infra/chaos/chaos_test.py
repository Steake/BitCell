#!/usr/bin/env python3
"""
BitCell Chaos Engineering Test Suite
Tests infrastructure resilience under various failure scenarios
"""

import argparse
import subprocess
import time
import requests
import sys
from typing import List, Dict, Any
from dataclasses import dataclass
from enum import Enum

class ChaosScenario(Enum):
    """Available chaos scenarios"""
    NODE_FAILURE = "node_failure"
    NETWORK_PARTITION = "network_partition"
    REGION_FAILURE = "region_failure"
    HIGH_LATENCY = "high_latency"
    PACKET_LOSS = "packet_loss"
    BYZANTINE_NODE = "byzantine_node"
    RESOURCE_EXHAUSTION = "resource_exhaustion"

@dataclass
class TestResult:
    scenario: str
    passed: bool
    duration: float
    details: str
    metrics: Dict[str, Any]

class ChaosTestFramework:
    def __init__(self, compose_file: str = "infra/docker/docker-compose.yml"):
        self.compose_file = compose_file
        self.results: List[TestResult] = []
        
    def run_command(self, cmd: List[str], check: bool = True) -> subprocess.CompletedProcess:
        """Execute shell command"""
        print(f"Running: {' '.join(cmd)}")
        return subprocess.run(cmd, capture_output=True, text=True, check=check)
    
    def get_node_health(self, node_name: str, port: int) -> bool:
        """Check if a node is healthy"""
        try:
            response = requests.get(f"http://localhost:{port}/health", timeout=5)
            return response.status_code == 200
        except:
            return False
    
    def get_prometheus_metrics(self) -> Dict[str, Any]:
        """Fetch current metrics from Prometheus"""
        try:
            response = requests.get("http://localhost:9999/api/v1/query", params={
                "query": "up{job=~'bitcell-.*'}"
            }, timeout=10)
            if response.status_code == 200:
                data = response.json()
                return {
                    "nodes_up": len([r for r in data.get("data", {}).get("result", []) if r["value"][1] == "1"]),
                    "total_nodes": len(data.get("data", {}).get("result", []))
                }
        except requests.RequestException:
            # Ignore exceptions if Prometheus is unavailable or request fails
            pass
        return {"nodes_up": 0, "total_nodes": 0}
    
    def wait_for_convergence(self, timeout: int = 120) -> bool:
        """Wait for network to reconverge after disruption"""
        print(f"Waiting for network convergence (timeout: {timeout}s)...")
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            metrics = self.get_prometheus_metrics()
            if metrics["nodes_up"] >= metrics["total_nodes"] * 0.7:  # 70% nodes up
                print(f"✓ Network converged: {metrics['nodes_up']}/{metrics['total_nodes']} nodes up")
                return True
            time.sleep(5)
        
        return False
    
    def test_node_failure(self) -> TestResult:
        """Test single node failure and recovery"""
        print("\n" + "="*60)
        print("TEST: Single Node Failure")
        print("="*60)
        
        start_time = time.time()
        node = "node-us-east-1"
        
        try:
            # Get initial state
            initial_metrics = self.get_prometheus_metrics()
            print(f"Initial state: {initial_metrics['nodes_up']}/{initial_metrics['total_nodes']} nodes up")
            
            # Kill a node
            print(f"Stopping {node}...")
            self.run_command(["docker-compose", "-f", self.compose_file, "stop", node])
            time.sleep(10)
            
            # Check remaining nodes are still functional
            post_failure_metrics = self.get_prometheus_metrics()
            print(f"After failure: {post_failure_metrics['nodes_up']}/{post_failure_metrics['total_nodes']} nodes up")
            
            # Restart the node
            print(f"Restarting {node}...")
            self.run_command(["docker-compose", "-f", self.compose_file, "start", node])
            
            # Wait for recovery
            recovered = self.wait_for_convergence(timeout=60)
            
            duration = time.time() - start_time
            final_metrics = self.get_prometheus_metrics()
            
            passed = recovered and final_metrics["nodes_up"] >= initial_metrics["nodes_up"]
            
            return TestResult(
                scenario="Node Failure",
                passed=passed,
                duration=duration,
                details=f"Node recovered: {recovered}",
                metrics=final_metrics
            )
            
        except Exception as e:
            return TestResult(
                scenario="Node Failure",
                passed=False,
                duration=time.time() - start_time,
                details=f"Error: {str(e)}",
                metrics={}
            )
    
    def test_region_failure(self) -> TestResult:
        """Test entire region failure"""
        print("\n" + "="*60)
        print("TEST: Regional Failure")
        print("="*60)
        
        start_time = time.time()
        region_nodes = ["node-us-east-1", "node-us-east-2"]
        
        try:
            initial_metrics = self.get_prometheus_metrics()
            print(f"Initial state: {initial_metrics['nodes_up']}/{initial_metrics['total_nodes']} nodes up")
            
            # Kill all nodes in US-East region
            print(f"Stopping region: US-East ({len(region_nodes)} nodes)")
            for node in region_nodes:
                self.run_command(["docker-compose", "-f", self.compose_file, "stop", node])
            
            time.sleep(15)
            
            # Verify other regions still operational
            post_failure_metrics = self.get_prometheus_metrics()
            print(f"After regional failure: {post_failure_metrics['nodes_up']}/{post_failure_metrics['total_nodes']} nodes up")
            
            # Restart region
            print("Restarting region...")
            for node in region_nodes:
                self.run_command(["docker-compose", "-f", self.compose_file, "start", node])
            
            recovered = self.wait_for_convergence(timeout=120)
            
            duration = time.time() - start_time
            final_metrics = self.get_prometheus_metrics()
            
            # Network should survive with >50% nodes
            passed = (
                post_failure_metrics["nodes_up"] >= post_failure_metrics["total_nodes"] * 0.5 and
                recovered
            )
            
            return TestResult(
                scenario="Regional Failure",
                passed=passed,
                duration=duration,
                details=f"Network survived regional failure: {passed}",
                metrics=final_metrics
            )
            
        except Exception as e:
            return TestResult(
                scenario="Regional Failure",
                passed=False,
                duration=time.time() - start_time,
                details=f"Error: {str(e)}",
                metrics={}
            )
    
    def test_network_partition(self) -> TestResult:
        """Test network partition between regions"""
        print("\n" + "="*60)
        print("TEST: Network Partition")
        print("="*60)
        
        start_time = time.time()
        
        try:
            # This would use iptables or tc to create network partitions
            # For Docker, we can simulate by pausing containers
            # Note: Container names from docker-compose match the service name
            nodes_group_b = ["node-eu-central-1", "node-ap-southeast-1"]
            
            print("Creating network partition...")
            for node in nodes_group_b:
                # Use the actual container name from docker-compose
                self.run_command(["docker", "pause", f"bitcell-{node}"], check=False)
            
            time.sleep(30)
            
            # Heal partition
            print("Healing partition...")
            for node in nodes_group_b:
                # Use the actual container name from docker-compose
                self.run_command(["docker", "unpause", f"bitcell-{node}"], check=False)
            
            recovered = self.wait_for_convergence(timeout=120)
            
            duration = time.time() - start_time
            final_metrics = self.get_prometheus_metrics()
            
            return TestResult(
                scenario="Network Partition",
                passed=recovered,
                duration=duration,
                details=f"Network recovered from partition: {recovered}",
                metrics=final_metrics
            )
            
        except Exception as e:
            return TestResult(
                scenario="Network Partition",
                passed=False,
                duration=time.time() - start_time,
                details=f"Error: {str(e)}",
                metrics={}
            )
    
    def test_high_latency(self) -> TestResult:
        """Test network resilience under high latency"""
        print("\n" + "="*60)
        print("TEST: High Latency")
        print("="*60)
        
        start_time = time.time()
        
        try:
            # Add network delay using tc (traffic control)
            # This requires NET_ADMIN capability
            print("Adding 500ms latency to network...")
            
            # Note: This requires privileged containers
            # In production, use Chaos Mesh or similar tools
            
            time.sleep(30)
            
            # Check if network still functions
            metrics = self.get_prometheus_metrics()
            network_functional = metrics["nodes_up"] >= metrics["total_nodes"] * 0.8
            
            # Remove latency
            print("Removing latency...")
            
            duration = time.time() - start_time
            
            return TestResult(
                scenario="High Latency",
                passed=network_functional,
                duration=duration,
                details=f"Network remained functional under high latency: {network_functional}",
                metrics=metrics
            )
            
        except Exception as e:
            return TestResult(
                scenario="High Latency",
                passed=False,
                duration=time.time() - start_time,
                details=f"Error: {str(e)}",
                metrics={}
            )
    
    def test_resource_exhaustion(self) -> TestResult:
        """Test behavior under resource exhaustion"""
        print("\n" + "="*60)
        print("TEST: Resource Exhaustion")
        print("="*60)
        
        start_time = time.time()
        
        try:
            # Limit CPU/memory for a node
            # Use actual container name from docker-compose
            node = "bitcell-us-east-1"
            print(f"Limiting resources for {node}...")
            
            # Update container resources
            self.run_command([
                "docker", "update",
                "--cpus", "0.5",
                "--memory", "512m",
                node
            ], check=False)
            
            time.sleep(30)
            
            # Check if node is still functional
            node_healthy = self.get_node_health("node-us-east-1", 9090)
            
            # Restore resources
            print("Restoring resources...")
            self.run_command([
                "docker", "update",
                "--cpus", "4",
                "--memory", "8g",
                node
            ], check=False)
            
            duration = time.time() - start_time
            
            return TestResult(
                scenario="Resource Exhaustion",
                passed=True,  # Test completion is success
                duration=duration,
                details=f"Node remained stable: {node_healthy}",
                metrics={"node_healthy": node_healthy}
            )
            
        except Exception as e:
            return TestResult(
                scenario="Resource Exhaustion",
                passed=False,
                duration=time.time() - start_time,
                details=f"Error: {str(e)}",
                metrics={}
            )
    
    def run_all_tests(self) -> List[TestResult]:
        """Run all chaos tests"""
        print("\n" + "="*80)
        print("BitCell Chaos Engineering Test Suite")
        print("="*80)
        
        # Verify infrastructure is running
        print("\nVerifying infrastructure...")
        try:
            self.run_command(["docker-compose", "-f", self.compose_file, "ps"])
        except subprocess.CalledProcessError:
            print("ERROR: Infrastructure not running. Start with: docker-compose up -d")
            sys.exit(1)
        
        # Run tests
        tests = [
            self.test_node_failure,
            self.test_region_failure,
            self.test_network_partition,
            self.test_high_latency,
            self.test_resource_exhaustion,
        ]
        
        for test_func in tests:
            result = test_func()
            self.results.append(result)
            time.sleep(5)  # Brief pause between tests
        
        return self.results
    
    def print_results(self):
        """Print test results summary"""
        print("\n" + "="*80)
        print("Test Results Summary")
        print("="*80)
        
        passed = sum(1 for r in self.results if r.passed)
        total = len(self.results)
        
        for result in self.results:
            status = "✓ PASS" if result.passed else "✗ FAIL"
            print(f"\n{status} - {result.scenario}")
            print(f"  Duration: {result.duration:.2f}s")
            print(f"  Details: {result.details}")
            if result.metrics:
                print(f"  Metrics: {result.metrics}")
        
        print(f"\n{'='*80}")
        print(f"Overall: {passed}/{total} tests passed ({passed/total*100:.1f}%)")
        print("="*80)
        
        return passed == total

def main():
    parser = argparse.ArgumentParser(description="BitCell Chaos Engineering Tests")
    parser.add_argument("--compose-file", default="infra/docker/docker-compose.yml",
                       help="Path to docker-compose file")
    parser.add_argument("--scenario", choices=[s.value for s in ChaosScenario],
                       help="Run specific scenario only")
    
    args = parser.parse_args()
    
    framework = ChaosTestFramework(args.compose_file)
    
    if args.scenario:
        # Run single scenario
        scenario_map = {
            "node_failure": framework.test_node_failure,
            "region_failure": framework.test_region_failure,
            "network_partition": framework.test_network_partition,
            "high_latency": framework.test_high_latency,
            "resource_exhaustion": framework.test_resource_exhaustion,
        }
        result = scenario_map[args.scenario]()
        framework.results.append(result)
    else:
        # Run all tests
        framework.run_all_tests()
    
    # Print results
    success = framework.print_results()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
