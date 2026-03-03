terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

provider "google" {
  project = var.project
  region  = var.region
  zone    = var.zone
}

resource "google_compute_network" "example" {
  name                    = "${var.instance_name}-network"
  auto_create_subnetworks = false
}

resource "google_compute_subnetwork" "example" {
  name          = "${var.instance_name}-subnet"
  ip_cidr_range = "10.0.1.0/24"
  network       = google_compute_network.example.id
}

resource "google_compute_instance" "example" {
  name         = var.instance_name
  machine_type = var.machine_type

  boot_disk {
    initialize_params {
      image = "debian-cloud/debian-12"
    }
  }

  network_interface {
    subnetwork = google_compute_subnetwork.example.id
  }
}
