output "instance_id" {
  description = "ID of the EC2 instance"
  value       = aws_instance.example.id
}

output "private_ip" {
  description = "Private IP address of the EC2 instance"
  value       = aws_instance.example.private_ip
}

output "ami_id" {
  description = "AMI ID used for the instance"
  value       = data.aws_ami.amazon_linux.id
}

output "vpc_id" {
  description = "VPC ID"
  value       = aws_vpc.example.id
}

output "subnet_id" {
  description = "Subnet ID"
  value       = aws_subnet.example.id
}
