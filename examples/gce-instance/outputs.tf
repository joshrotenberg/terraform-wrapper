output "instance_id" {
  description = "ID of the GCE instance"
  value       = google_compute_instance.example.instance_id
}

output "internal_ip" {
  description = "Internal IP address of the instance"
  value       = google_compute_instance.example.network_interface[0].network_ip
}

output "network_id" {
  description = "Network ID"
  value       = google_compute_network.example.id
}

output "subnet_id" {
  description = "Subnet ID"
  value       = google_compute_subnetwork.example.id
}
