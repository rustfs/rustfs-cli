rustfs 桶复制用到的接口：


### SetRemoteTarget- 非s3 标准协议

----- REQUEST -----

Method: PUT

URL: http://0.0.0.0:9000/minio/admin/v3/set-remote-target?bucket=srcbucket

Headers:

  Host: 0.0.0.0:9000
  
  User-Agent: MinIO (linux; amd64) madmin-go/3.0.70 mc/DEVELOPMENT.2025-05-06T16-41-33Z
  
  Content-Length: 527
  
  Accept-Encoding: zstd,gzip
  
  Authorization: AWS4-HMAC-SHA256 Credential=rustfsadmin/20250626//s3/aws4_request, SignedHeaders=host;x
-amz-content-sha256;x-amz-date, Signature=9bee3c0db82f3fef71fe316fe7d47dc92124e1f9b22917171a1790b8249844
76

  X-Amz-Content-Sha256: 5b83787f836d2f193f883f9f3de442f1a518897fb898860e1bf5402a0f7e14d7
  
  X-Amz-Date: 20250626T165122Z
  
Body:

{"sourcebucket":"","endpoint":"0.0.0.0:9001","credentials":{"accessKey":"rustfsadmin","secretKey":"rustf
sadmin","expiration":"0001-01-01T00:00:00Z"},"targetbucket":"destbucket","secure":false,"path":"auto","a
pi":"s3v4","type":"replication","replicationSync":false,"healthCheckDuration":60000000000,"disableProxy"
:false,"resetBeforeDate":"0001-01-01T00:00:00Z","totalDowntime":0,"lastOnline":"0001-01-01T00:00:00Z","i
sOnline":false,"latency":{"curr":0,"avg":0,"max":0},"edge":false,"edgeSyncBeforeExpiry":false,"offlineCo
unt":0}

----- RESPONSE -----

Status: 200

Headers:

  vary: origin, access-control-request-method, access-control-request-headers
  access-control-allow-origin: *
  access-control-expose-headers: *
  content-length: 81
  date: Thu, 26 Jun 2025 16:51:22 GMT
  
Body:

"arn:minio:replication:us-east-1:8dec15ee-2bb9-4b68-a1a6-59decfbf33ec:destbucket"
### deletebucketreplication-标准接口， 参考 aws api 文档即可；
https://docs.aws.amazon.com/AmazonS3/latest/API/API_DeleteBucketReplication.html



### bucket location-标准接口， 参考 aws api 文档即可

https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketLocation.html



### putbucketreplication-标准接口，参考 aws api 文档即可

https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketReplication.html




### getbucketreplicaiton-标准接口，参考 aws api 文档即可

https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketReplication.html 










### ListRemoteTargets-非 s3 标准协议

----- REQUEST -----

Method: GET

URL: http://0.0.0.0:9000/minio/admin/v3/list-remote-targets?bucket=srcbucket&type=

Headers:

  Host: 0.0.0.0:9000
  
  User-Agent: MinIO (linux; amd64) madmin-go/3.0.70 mc/DEVELOPMENT.2025-05-06T16-41-33Z
  
  Accept-Encoding: zstd,gzip
  
  Authorization: AWS4-HMAC-SHA256 Credential=rustfsadmin/20250626//s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=ff38b6e8056ab738e89e10601d10c36d468e965b6d1900fb7230e9f49f362474
  
  X-Amz-Content-Sha256: e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
  
  X-Amz-Date: 20250626T162557Z
  
Body:

----- RESPONSE -----

Status: 200

Headers:

  vary: origin, access-control-request-method, access-control-request-headers
  access-control-allow-origin: *
  access-control-expose-headers: *
  content-length: 708
  date: Thu, 26 Jun 2025 16:25:57 GMT
  
Body:

[
  {
    "sourcebucket": "srcbucket",
    "endpoint": "0.0.0.0:9001",
    "credentials": {
      "accessKey": "rustfsadmin",
      "secretKey": "rustfsadmin",
      "session_token": null,
      "expiration": "0001-01-01T00:00:00Z"
    },
    "targetbucket": "destbucket",
    "secure": false,
    "path": "auto",
    "api": "s3v4",
    "arn": "arn:minio:replication:us-east-1:8dec15ee-2bb9-4b68-a1a6-59decfbf33ec:destbucket",
    "type": "replication",
    "region": null,
    "bandwidth_limit": null,
    "replicationSync": false,
    "storage_class": null,
    "healthCheckDuration": 60000000000,
    "disableProxy": false,
    "resetBeforeDate": "0001-01-01T00:00:00Z",
    "reset_id": null,
    "totalDowntime": 0,
    "last_online": null,
    "isOnline": false,
    "latency": {
      "curr": 0,
      "avg": 0,
      "max": 0
    },
    "deployment_id": null,
    "edge": false,
    "edgeSyncBeforeExpiry": false
  }
]

