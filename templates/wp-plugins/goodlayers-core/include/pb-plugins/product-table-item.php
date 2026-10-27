<?php
	/*	
	*	Goodlayers Product Item
	*/
	
	if( !class_exists('gdlr_core_product_table_item') ){
		class gdlr_core_product_table_item{
			
			var $settings = '';
			
			// init the variable
			function __construct( $settings = array() ){
				
				$this->settings = wp_parse_args($settings, array(
					'category' => '', 
					'tag' => '', 
					'stock-status' => 'all', 
					'num-fetch' => '9', 
					'orderby' => 'date', 
					'order' => 'desc',
					'pagination' => 'none'
				));

				if( $this->settings['orderby'] == 'rand' ){
					$this->settings['orderby'] = 'RAND(' . rand(1, 1000) . ')';
				} 
			}
			
			// get the content of the product item
			function get_content(){

				$ret = '';
				if( empty($this->settings['query']) ){
					$query = $this->get_product_query();
				}else{
					$query = $this->settings['query'];
				}

				gdlr_core_setup_admin_postdata();

				if( !empty($this->settings['enable-category-filter']) && $this->settings['enable-category-filter'] == 'enable' ){
					$extra_class  = ' gdlr-core-item-pdlr';
					$extra_class .= ' gdlr-core-style-text';
					$extra_class .= empty($this->settings['filterer-align'])? '': ' gdlr-core-' . $this->settings['filterer-align'] . '-align';
					$extra_class .= apply_filters('gdlr_core_blog_filterer_class', '', $this->settings);

					$this->settings['filterer-slide-bar'] = 'enable';

					$ret .= gdlr_core_get_ajax_filterer('product_table', 'product_cat', $this->settings, 'gdlr-core-product-item-table-holder', $extra_class);
				}
				
				$ret .= '<div class="gdlr-core-product-item-table-holder gdlr-core-js-2 clearfix" >';
				$ret .= $this->get_product_table_content($query);
				$ret .= '</div>';

				wp_reset_postdata();
				gdlr_core_reset_admin_postdata();

				// pagination
				/*
				if( $this->settings['pagination'] != 'none' ){
					$extra_class = 'gdlr-core-item-pdlr';

					if( $this->settings['pagination'] == 'page' ){
						$ret .= gdlr_core_get_pagination($query->max_num_pages, $this->settings, $extra_class);
					}else if( $this->settings['pagination'] == 'load-more' ){
						$paged = empty($query->query['paged'])? 2: intval($query->query['paged']) + 1;
						$ret .= gdlr_core_get_ajax_load_more('product', $this->settings, $paged, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
					}
				}
				*/

				return $ret;
			}

			function get_product_attribute( $key ){
				global $product;
				return $product->get_attribute($key);
			}
			function get_product_attribute_variation(){
				global $product; 

				$ret = array();
				if( $product->get_type() == 'variable' ){

					$variation = $product->get_variation_prices(true);
					if( !empty($variation['price']) ){
						foreach( $variation['price'] as $variation_id => $price ){
							$wc_variation = new WC_Product_Variation($variation_id);
							$wc_variation_title = $wc_variation->get_title() . ' - ';

							$label = str_replace($wc_variation_title, '', $wc_variation->get_name());
							$ret[$label] = wc_price($price);
						}
					}
				}
				
				return $ret;
			}

			// get content of non carousel product item
			function get_product_table_content( $query ){

				$ret = '';
				$count = 0;

				$column_size = empty($this->settings['column'])? 1: intval($this->settings['column']);
				$column_item_size = ceil($query->post_count / $column_size);

				while($query->have_posts()){ $query->the_post(); $count++;

					if( $column_size > 1 && $count % $column_item_size ==  1 ){
						$ret .= '<div class="gdlr-core-column-' . esc_attr((60 / $column_size)) . '" >'; 
					}

					$ret .= '<div class="gdlr-core-item-list gdlr-core-item-pdlr" >';
					$ret .= '<div class="gdlr-core-item-list-inner" ' . gdlr_core_esc_style(array(
						'border-color' => empty($this->settings['divider-color'])? '': $this->settings['divider-color']
					)) . ' >';
					
					$ret .= '<div class="gdlr-core-product-table-head-wrap" ' . gdlr_core_esc_style(array(
						'flex-grow' => empty($this->settings['head-size'])? '': $this->settings['head-size']
					)) . ' >';
					$feature_image = get_post_thumbnail_id();

					if( empty($this->settings['with-thumbnail']) || $this->settings['with-thumbnail'] == 'enable' ){
						$ret .= '<div class="gdlr-core-product-thumbnail gdlr-core-media-image" >';
						$ret .= gdlr_core_get_image($feature_image, 'thumbnail', array('placeholder' => false));
						$ret .= '</div>';
					}
					
					$ret .= '<div class="gdlr-core-product-table-head" >';
					$ret .= '<h3 class="gdlr-core-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
						'font-size' => empty($this->settings['product-title-font-size'])? '': $this->settings['product-title-font-size'],
						'font-weight' => empty($this->settings['product-title-font-weight'])? '': $this->settings['product-title-font-weight'],
						'letter-spacing' => empty($this->settings['product-title-letter-spacing'])? '': $this->settings['product-title-letter-spacing'],
						'text-transform' => (empty($this->settings['product-title-text-transform']) || $this->settings['product-title-text-transform'] == 'none')? '': $this->settings['product-title-text-transform']
					)) . ' ><a href="' . get_permalink() . '" ' . gdlr_core_esc_style(array(
						'color' => empty($this->settings['title-color'])? '': $this->settings['title-color']
					)) . ' >' . get_the_title() . '</a></h3>';
					
					if( !empty($this->settings['attribute-fields']) ){
						$caption_fields = array_map('trim', explode('|', $this->settings['attribute-fields']));
						if( !empty($caption_fields) ){
							$ret .= '<div class="gdlr-core-caption" ' . gdlr_core_esc_style(array(
								'color' => empty($this->settings['attribute-color'])? '': $this->settings['attribute-color']
							)) . ' >';
							foreach( $caption_fields as $field ){
								$att = $this->get_product_attribute($field);
								$ret .= '<span class="gdlr-core-att" >' . $field . ': ' . $att . '</span>';
							} 
							$ret .= '</div>';
						}
					}
					$ret .= '</div>'; // gdlr-core-product-table-head
					$ret .= '</div>'; // gdlr-core-product-table-head-wrap

					if( !empty($this->settings['variation-fields']) ){
						$variations = $this->get_product_attribute_variation();
						$variation_fields =  array_map('trim', explode('|', $this->settings['variation-fields']));
						$ret .= '<div class="gdlr-core-product-variation-table" ' . gdlr_core_esc_style(array(
							'flex-grow' => empty($this->settings['variation-size'])? '': $this->settings['variation-size']
						)) . ' >';
						foreach( $variation_fields as $field_slug ){
							$ret .= '<div class="gdlr-core-item" >';
							if( !empty($variations[$field_slug]) ){
								$ret .= '<div class="gdlr-core-tail" ' . gdlr_core_esc_style(array(
									'color' => empty($this->settings['variation-price-color'])? '': $this->settings['variation-price-color']
								)) . ' >' . $variations[$field_slug] . '</div>';
								$ret .= '<div class="gdlr-core-head" ' . gdlr_core_esc_style(array(
									'color' => empty($this->settings['variation-color'])? '': $this->settings['variation-color']
								)) . ' >' . $field_slug . '</div>';
							}
							$ret .= '</div>';
						}
						$ret .= '</div>';
					}
					
					if( empty($this->settings['with-learn-more-button']) || $this->settings['with-learn-more-button'] == 'enable' ){		
						$ret .= '<div class="gdlr-core-learn-more" ><a href="' . get_permalink() . '" ' . gdlr_core_esc_style(array(
							'color' => empty($this->settings['read-more-color'])? '': $this->settings['read-more-color']
						)) . ' >';
						$ret .= esc_html__('Learn More', 'goodlayers-core');
						$ret .= '</a></div>';
					}
					
					$ret .= '</div>'; // gdlr-core-item-list-inner
					$ret .= '</div>'; // gdlr-core-item-list

					if( $column_size > 1 && $count % $column_item_size ==  0 ){
						$ret .= '</div>';
					}

				} // while

				if( $column_size > 1 && $count % $column_item_size !=  0 ){
					$ret .= '</div>';
				}

				return $ret;
			}
			
			// query the post
			function get_product_query(){
				
				$args = array( 'post_type' => 'product', 'post_status' => 'publish', 'suppress_filters' => false );
				
				// category - tag selection
				if( !empty($this->settings['category']) || !empty($this->settings['tag']) ){
					$args['tax_query'] = array(
						'relation' => empty($this->settings['relation'])? 'OR': $this->settings['relation']
					);
					
					if( !empty($this->settings['category']) ){
						array_push($args['tax_query'], array('terms'=>$this->settings['category'], 'taxonomy'=>'product_cat', 'field'=>'slug'));
					}
					if( !empty($this->settings['tag']) ){
						array_push($args['tax_query'], array('terms'=>$this->settings['tag'], 'taxonomy'=>'product_tag', 'field'=>'slug'));
					}
				}

				// pagination
				if( empty($this->settings['paged']) ){
					$args['paged'] = (get_query_var('paged'))? get_query_var('paged') : get_query_var('page');
					$args['paged'] = empty($args['paged'])? 1: $args['paged'];
				}else{
					$args['paged'] = $this->settings['paged'];
				}
				$this->settings['paged'] = $args['paged'];				

				if( !empty($this->settings['stock-status']) && $this->settings['stock-status'] != 'all' ){
					 $args['meta_query'] = array(
				        array(
				            'key' => '_stock_status',
				            'value' => $this->settings['stock-status']
				        )
				    );
				}

				// variable
				$args['posts_per_page'] = $this->settings['num-fetch'];
				$args['orderby'] = $this->settings['orderby'];
				$args['order'] = $this->settings['order'];
				
				$query = new WP_Query( $args );

				return $query;
			}
			
		} // gdlr_core_product_item
	} // class_exists
	
	add_action('wp_ajax_gdlr_core_product_table_ajax', 'gdlr_core_product_table_ajax');
	add_action('wp_ajax_nopriv_gdlr_core_product_table_ajax', 'gdlr_core_product_table_ajax');
	if( !function_exists('gdlr_core_product_table_ajax') ){
		function gdlr_core_product_table_ajax(){

			if( !empty($_POST['settings']) ){

				$settings = $_POST['settings'];
				if( !empty($_POST['option']) ){	
					$settings[$_POST['option']['name']] = $_POST['option']['value'];

					if( $_POST['option']['name'] == 'category' ){
						$settings['paged'] = 1;
					}
				}else{
					$settings['paged'] = 1;
				}

				$product_item = new gdlr_core_product_table_item($settings);
				$query = $product_item->get_product_query();	

				remove_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_price', 10 );
				remove_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_rating', 5 );

				$ret = array(
					'status'=> 'success',
					'content'=> $product_item->get_product_table_content($query)
				);

				add_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_price', 10 );
				add_action( 'woocommerce_after_shop_loop_item_title', 'woocommerce_template_loop_rating', 5 );
				
				/*
				if( !empty($settings['pagination']) && $settings['pagination'] != 'none' ){
					if( $settings['pagination'] == 'load-more' ){
						$paged = empty($query->query['paged'])? 2: intval($query->query['paged']) + 1;
						$extra_class = ($settings['no-space'] == 'yes')? '': 'gdlr-core-item-pdlr';

						$ret['load_more'] = gdlr_core_get_ajax_load_more('product', $settings, $paged, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
						$ret['load_more'] = empty($ret['load_more'])? 'none': $ret['load_more'];
					}else if( empty($_POST['option']['name']) || $_POST['option']['name'] == 'category' ){
						$ret['pagination'] = gdlr_core_get_ajax_pagination('product', $settings, $query->max_num_pages, 'gdlr-core-product-item-holder', $extra_class);
						$ret['pagination'] = empty($ret['pagination'])? 'none': $ret['pagination'];
					}
				} 
				*/

				die(json_encode($ret));
			}else{
				die(json_encode(array(
					'status'=> 'failed',
					'message'=> esc_html__('Settings variable is not defined.', 'goodlayers-core')
				)));
			}

		} // gdlr_core_product_load_more
	} // function_exists

	