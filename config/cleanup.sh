#!/bin/bash
kill -s SIGUSR1 $(systemctl show --property MainPID syncSlide.service | cut -d= -f2)
