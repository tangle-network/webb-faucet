# Copyright 2022 Webb Technologies Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
FROM gcr.io/distroless/static

LABEL APP="Webb Faucet"
LABEL AUTHOR="Webb Developers <dev@webb.tools>"

ENV RUST_BACKTRACE=full

VOLUME [ "/faucet" ]

WORKDIR /faucet

RUN mkdir -p /usr/bin || exit 0
ADD build/webb-faucet /usr/bin/webb-faucet

EXPOSE 8000

CMD ["/usr/bin/webb-faucet"]
