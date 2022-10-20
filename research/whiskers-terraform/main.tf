terraform  {
  required_providers {
    aws = {
      source = "hashicorp/aws"
      version = "~> 4.0"
    }
  }
}

provider "aws" {
  region = "us-east-2"
}

resource "aws_security_group" "allow_http" {
  name        = "allow_http"
  description = "Allow TLS inbound traffic"
  vpc_id      = "vpc-0eb60cd1e5650a5d9"

  ingress {
    description      = "Port 80"
    from_port        = 80 
    to_port          = 80 
    protocol         = "tcp"
    cidr_blocks      = ["0.0.0.0/0"]
    ipv6_cidr_blocks = ["::/0"]
  }

  egress {
    from_port        = 0
    to_port          = 0
    protocol         = "-1"
    cidr_blocks      = ["0.0.0.0/0"]
    ipv6_cidr_blocks = ["::/0"]
  }

  tags = {
    Name = "port_80"
  }
}

resource "aws_instance" "whiskers" {
  ami = "ami-0bde60638be9bb870"
  instance_type = "t3.micro"
  subnet_id = "subnet-07d580fee7a806230"
  vpc_security_group_ids = [aws_security_group.allow_http.id]
  key_name = "si_key"
  associate_public_ip_address = true

  user_data = <<EOT
{
  "ignition": {
    "version": "3.3.0"
  },
  "systemd": {
    "units": [
      {
        "contents": "[Unit]\nDescription=Whiskers\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill whiskers1\nExecStartPre=-/bin/podman rm whiskers1\nExecStartPre=/bin/podman pull docker.io/systeminit/whiskers\nExecStart=/bin/podman run --name whiskers1 --publish 80:80 docker.io/systeminit/whiskers\n\n[Install]\nWantedBy=multi-user.target\n",
        "enabled": true,
        "name": "whiskers.service"
      }
    ]
  }
}
  EOT

  tags = {
    Name = "whiskers"
    Environment = "prod"
    Terraform = "true"
  }
}

