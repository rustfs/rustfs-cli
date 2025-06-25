rustfs 桶复制用到的接口：

### getbucketreplicaiton-标准接口，参考 aws api 文档即可

https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetBucketReplication.html 

### putbucketreplication-标准接口，参考 aws api 文档即可

https://docs.aws.amazon.com/AmazonS3/latest/API/API_PutBucketReplication.html

### SetRemoteTarget- 非s3 标准协议

method: PUT

URL and PAR: http://127.0.0.1:7000/rustfs/admin/v3/set-remote-target?bucket=$bucket

PUT body:
json: 


### ListRemoteTargets-非 s3 标准协议

method: GET

URL and PAR:  http://127.0.0.1:7000/rustfs/admin/v3/list-remote-targets?bucket=$bucket

response data：
