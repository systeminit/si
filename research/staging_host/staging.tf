terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
  }
}

provider "aws" {
  region = "us-east-2"
}

data "local_file" "ignition" {
  filename = "${path.module}/staging-1.ign"
}

resource "aws_instance" "staging-1" {
  ami                    = "ami-0e6f4ffb61e585c76"
  instance_type          = "t3.large"
  subnet_id              = "subnet-07d580fee7a806230"
  vpc_security_group_ids = ["sg-0d0be672e4485feb4"]
  key_name               = "si_key"
  iam_instance_profile   = "veritech-ec2"

  user_data = data.local_file.ignition.content

  tags = {
    Name        = "staging-1"
    Environment = "staging"
    Terraform   = "true"
  }
}

resource "aws_eip_association" "eip_association" {
  instance_id   = aws_instance.staging-1.id
  allocation_id = "eipalloc-0f8bdc206768cb6a7"
}