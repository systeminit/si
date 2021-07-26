terraform {
  required_version = ">= 1.0.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 3.46"
    }
  }
}

variable "region" {
  default = "us-east-2"
}

variable "ami" {
  default = "ami-008ef391bf64d8b7f"
}

variable "instance_type" {
  default = "t2.xlarge"
}

variable "key_name" {
  default = "si@aws-2021-07-19T22:56:24Z"
}

provider "aws" {
  region = var.region
}

data "aws_vpc" "default" {
  default = true
}

data "aws_subnet_ids" "default" {
  vpc_id = data.aws_vpc.default.id
}

resource "aws_security_group" "remote_access" {
  name        = "remote-access"
  description = "Allow SSH and Mosh remote access traffic"
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  ingress {
    from_port   = "60000"
    to_port     = "60010"
    protocol    = "udp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "vpc_web" {
  name        = "vpc-web"
  description = "Allow web traffic from VPC"
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.default.cidr_block]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "lb_web" {
  name        = "lb-web"
  description = "Allow web traffic to the load balancer"
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_instance" "si_dev" {
  ami                         = var.ami
  instance_type               = var.instance_type
  associate_public_ip_address = true
  root_block_device {
    volume_size = 40
  }
  key_name               = var.key_name
  vpc_security_group_ids = [aws_security_group.remote_access.id, aws_security_group.vpc_web.id]
  user_data              = file("init.sh")
}

resource "aws_lb_target_group" "si_dev" {
  name     = "si-lb-tg"
  port     = 80
  protocol = "HTTP"
  vpc_id   = data.aws_vpc.default.id
}

resource "aws_lb_target_group_attachment" "si_dev" {
  target_group_arn = aws_lb_target_group.si_dev.arn
  target_id        = aws_instance.si_dev.id
  port             = 80
}

resource "aws_lb" "si_dev" {
  name               = "si-dev-lb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.lb_web.id]
  subnets            = data.aws_subnet_ids.default.ids
}

resource "aws_lb_listener" "si_dev" {
  load_balancer_arn = aws_lb.si_dev.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.si_dev.arn
  }
}

output "ssh_user" {
  value = "si"
}

output "si_dev_public_dns" {
  value = aws_instance.si_dev.public_dns
}

output "si_dev_public_ip" {
  value = aws_instance.si_dev.public_ip
}

output "lb_dns_name" {
  value = aws_lb.si_dev.dns_name
}

