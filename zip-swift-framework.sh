#!/bin/sh

set -eo pipefail

echo "Zipping anoncreds_uniffiFFI.xcframework"
zip -rq anoncreds/out/anoncreds_uniffiFFI.xcframework.zip anoncreds/out/anoncreds_uniffiFFI.xcframework
swift package compute-checksum anoncreds/out/anoncreds_uniffiFFI.xcframework.zip

echo "Zipping askar_uniffiFFI.xcframework"
zip -rq askar/out/askar_uniffiFFI.xcframework.zip askar/out/askar_uniffiFFI.xcframework
swift package compute-checksum askar/out/askar_uniffiFFI.xcframework.zip

echo "Zipping indy-vdr_uniffiFFI.xcframework"
zip -rq indy-vdr/out/indy_vdr_uniffiFFI.xcframework.zip indy-vdr/out/indy_vdr_uniffiFFI.xcframework
swift package compute-checksum indy-vdr/out/indy_vdr_uniffiFFI.xcframework.zip
