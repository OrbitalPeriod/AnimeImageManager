group "default" {
  targets = ["tagmonolith"]
}

target "tagmonolith" {
  context = "."
  dockerfile = "Dockerfile"
  tags = ["tagmonolith:latest"]
}
