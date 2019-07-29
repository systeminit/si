# -*- mode: ruby -*-
# vi: set ft=ruby :

APP_NAME = "bugbear"

Vagrant.configure("2") do |config|

  config.vm.define APP_NAME
  config.vm.box = "ubuntu/bionic64" #18.04 LTS
  config.vm.hostname = APP_NAME
  config.vm.network "forwarded_port", guest: 4000, host: 4000
  config.vm.network "forwarded_port", guest: 8080, host: 8080
  config.vm.network "public_network", bridge: "en0: Wi-Fi (Wireless)"
  config.vm.provider "virtualbox" do |vb|
    vb.memory = "2048"
    vb.name = APP_NAME
  end
  config.vm.synced_folder "./", "/app"
  config.vm.provision "shell", path: "./scripts/vagrant-provision.sh"

end

