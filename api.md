curl -X GET "http://127.0.0.1:9000/minio/admin/v3/list-remote-targets?bucket=<bucket>&type=<arnType>" \
   -H "Authorization: Bearer" \
   -H "Content-Type: application/json"
