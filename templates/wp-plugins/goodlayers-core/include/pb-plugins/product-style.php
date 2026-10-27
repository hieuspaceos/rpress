<?php
	/*	
	*	Goodlayers Product Item Style
	*/
	
	if( !class_exists('gdlr_core_product_style') ){
		class gdlr_core_product_style{

			public $product_item_id;

			function __construct(){

				global $product_item_id;
				$product_item_id = empty($product_item_id)? 1: $product_item_id+1;
				$this->product_item_id = $product_item_id;
			}

			// get the content of the product item
			function get_content( $args ){

				$ret = apply_filters('gdlr_core_product_style_content', '', $args, $this);
				if( !empty($ret) ) return $ret;

				switch( $args['product-style'] ){
					case 'grid': 
						return $this->product_grid( $args ); 
					case 'grid-2': 
						return $this->product_grid_style_2( $args ); 
					case 'grid-3': 
					case 'grid-3-with-border': 
					case 'grid-3-without-frame': 
						return $this->product_grid_style_3( $args ); 
					case 'grid-4': 
						return $this->product_grid_style_4( $args ); 
					case 'grid-5': 
						return $this->product_grid_style_5( $args ); 
					case 'grid-6': 
						return $this->product_grid_style_6( $args ); 
					case 'box': 
						return $this->product_box( $args ); 
					break;
				}
				
			}

			// get product thumbnail
			function product_thumbnail( $args = array() ){
				$ret = '';
				$feature_image = get_post_thumbnail_id();

				if( !empty($feature_image) ){
					$additional_class = '';
					if( empty($args['enable-thumbnail-zoom-on-hover']) || $args['enable-thumbnail-zoom-on-hover'] == 'enable' ){
						$additional_class .= ' gdlr-core-zoom-on-hover';
					}
					if( !empty($args['enable-thumbnail-grayscale-effect']) && $args['enable-thumbnail-grayscale-effect'] == 'enable' ){
						$additional_class .= ' gdlr-core-grayscale-effect';
					}

					$shadow_atts = array();
					if( !in_array($args['product-style'], array('grid-3', 'grid-3-with-border', 'grid-5')) ){
						if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
							$shadow_atts = array(
								'background-shadow-size' => $args['frame-shadow-size'],
								'background-shadow-color' => $args['frame-shadow-color'],
								'background-shadow-opacity' => $args['frame-shadow-opacity'],
							);

							$additional_class .= ' gdlr-core-outer-frame-element';
						}
					}


					$ret .= '<div class="gdlr-core-product-thumbnail gdlr-core-media-image ' . esc_attr($additional_class) . '" ';
					$ret .= gdlr_core_esc_style($shadow_atts);
					$ret .= ' >';

					$thumbnail_link = (empty($args['enable-thumbnail-link']) || $args['enable-thumbnail-link'] == 'enable')? true: false;
					if( in_array($args['product-style'], array('grid', 'grid-2')) ){
						$thumbnail_link = false;
					}else if( $args['product-style'] == 'grid-3' && !empty($args['button-style']) && $args['button-style'] == 'thumbnail-modern' ){
						$thumbnail_link = false;
					}
					if( $thumbnail_link ){
						$ret .= '<a href="' . get_permalink() . '" >';
					}
					$ret .= gdlr_core_get_image($feature_image, $args['thumbnail-size'], array('placeholder' => false));
					
					global $product;
					if( !$product->is_in_stock()){
						$ret .= '<span class="gdlr-core-wc-out-of-stock">' . esc_html__('Out Of Stock', 'goodlayers-core') . '</span>';
					}
					
					if( $args['product-style'] == 'grid' ){
						$ret .= '<div class="gdlr-core-product-thumbnail-info" >';
						$ret .= '<a href="' . get_permalink() . '" class="gdlr-core-product-view-detail" >';
						$ret .= '<i class="fa fa-eye" ></i>';
						$ret .= '<span>' . esc_html__('View Details', 'goodlayers-core') . '</span>';
						$ret .= '</a>';					
						$ret .= $this->product_add_to_cart();
						$ret .= '</div>';
					}else if( $args['product-style'] == 'grid-2' ){
						$grid2_thumbnail_class = '';
						$button_start_html = '<i class="icon_cart_alt" ></i><span>';
						if( !empty($args['price-location']) && $args['price-location'] == 'thumbnail' ){
							$grid2_thumbnail_class = 'gdlr-core-with-price';
							$button_start_html = $this->product_price() . '<span>';
						}

						$ret .= '<div class="gdlr-core-product-thumbnail-info ' . esc_attr($grid2_thumbnail_class) . '" >';
						$ret .= $this->product_add_to_cart($button_start_html);
						$ret .= '</div>';
					}else if( $args['product-style'] == 'grid-3' && !empty($args['button-style']) && $args['button-style'] == 'thumbnail-modern' ){
						$ret .= $this->product_onsale();
						$ret .= '<div class="gdlr-core-product-thumbnail-info-modern" >';
						$ret .= '<div class="gdlr-core-product-thumbnail-info-modern-inner" >';
						if( method_exists( 'YITH_WCWL_Shortcode', 'add_to_wishlist' ) ){
							$ret .= YITH_WCWL_Shortcode::add_to_wishlist(
								array('product_id' => get_the_ID())
							);
						}
						
						$ret .= $this->product_add_to_cart();
						$ret .= '<a class="gdlr-core-product-thumbnail-link" href="' . esc_url(get_permalink()) .  '" ><i class="fa fa-eye" ></i></a>';
						$ret .= '</div>';
						$ret .= '</div>';
					}else{
						if( $args['product-style'] == 'grid-6' || $args['product-style'] == 'grid-3' ){
							$ret .= $this->product_onsale();
						}
					}
					if( $thumbnail_link ){
						$ret .= '</a>';
					}

					$ret .= '</div>';
				}

				return $ret;
			}

			// get product add to cart button
			function product_add_to_cart( $start = '<i class="icon_cart_alt" ></i><span>', $end = '</span>' ){

				add_filter('woocommerce_loop_add_to_cart_args', array($this, 'product_add_to_cart_args'));
				add_filter('woocommerce_product_add_to_cart_text', array($this, 'product_add_to_cart_text'));

				ob_start();
				woocommerce_template_loop_add_to_cart();
				$ret = ob_get_contents();
				ob_end_clean();

				remove_filter('woocommerce_loop_add_to_cart_args', array($this, 'product_add_to_cart_args'));
				remove_filter('woocommerce_product_add_to_cart_text', array($this, 'product_add_to_cart_text'));

				// replace the text
				$ret = str_replace('#gdlr-core-product-add-to-cart#', $start, $ret);
				$ret = str_replace('#gdlr-core-product-add-to-cart-end#', $end, $ret);

				return $ret;
			}
			function product_add_to_cart_args( $args ){

				$args['class'] = preg_replace('/^button\s/', '', $args['class']);
				$args['class'] .= ' gdlr-core-product-add-to-cart';

				return $args;
			}
			function product_add_to_cart_text( $text ){
				return '#gdlr-core-product-add-to-cart#' . $text . '#gdlr-core-product-add-to-cart-end#';
			}

			// get product price
			function product_price(){
				global $product;

				$ret = '';
				if( !empty($product) ){
					$ret  = '<div class="gdlr-core-product-price gdlr-core-title-font">';
					$ret .= $product->get_price_html();
					$ret .= '</div>';
				}

				return $ret;
			}
			function product_rating(){
				global $product;

				$ret = '';
				if( !empty($product) ){
					$average = $product->get_average_rating();
					$rating_count = $product->get_rating_count();
					$ret .= wc_get_rating_html($average, $rating_count);
				}

				return $ret;
			}
			function product_rating_with_count(){
				global $product;

				$ret = '';
				if( !empty($product) ){
					$average = $product->get_average_rating();
					$rating_count = $product->get_rating_count();

					if( $rating_count > 0 ){
						$ret .= '<div class="gdlr-core-rating" >';
						$ret .= wc_get_rating_html($average, $rating_count);
						if( $rating_count <= 1 ){
							$ret .= '<span class="gdlr-core-rating-count" >' . sprintf(esc_html__('(%d Review)', 'goodlayers-core'), $rating_count) . '</span>';
						}else{
							$ret .= '<span class="gdlr-core-rating-count" >' . sprintf(esc_html__('(%d Reviews)', 'goodlayers-core'), $rating_count) . '</span>';
						}
						$ret .= '</div>';
					}
					
				}

				return $ret;
			}
			function product_onsale(){
				global $product;

				$ret = '';
				if( !empty($product) ){
					ob_start();
					woocommerce_show_product_loop_sale_flash();
					$ret = ob_get_contents();
					ob_end_clean();

					if( !empty($ret) ){
						$ret .= '<span class="gdlr-core-outer-frame-element" ></span>';
					}
				}
				
				return $ret;
			}

			// product title
			function product_title( $args, $permalink = '' ){

				$ret  = '<h3 class="gdlr-core-product-title gdlr-core-skin-title" ' . gdlr_core_esc_style(array(
					'font-size' => empty($args['product-title-font-size'])? '': $args['product-title-font-size'],
					'font-weight' => empty($args['product-title-font-weight'])? '': $args['product-title-font-weight'],
					'letter-spacing' => empty($args['product-title-letter-spacing'])? '': $args['product-title-letter-spacing'],
					'text-transform' => (empty($args['product-title-text-transform']) || $args['product-title-text-transform'] == 'none')? '': $args['product-title-text-transform']
				)) . ' >';
				if( empty($permalink) ){
					$ret .= '<a href="' . get_permalink() . '" ' . gdlr_core_esc_style(array(
						'color' => empty($args['product-title-color'])? '': $args['product-title-color']
					)) . ' >'; 
				}else{
					$ret .= '<a href="' . esc_attr($permalink) . '" target="_blank" ' . gdlr_core_esc_style(array(
						'color' => empty($args['product-title-color'])? '': $args['product-title-color']
					)) . ' >';
				}
				$ret .= get_the_title();
				$ret .= '</a>';
				$ret .= '</h3>';
				
				ob_start();
				do_action('woocommerce_after_shop_loop_item_title'); 
				$ret .= ob_get_contents();
				ob_end_clean();


				return  $ret;
			}

			// product attributes
			function product_attributes( $args ){

				if( empty($args['display-attribute-amount']) ) return; 
				$count = intval($args['display-attribute-amount']);

				global $product;
				$attributes = array_filter($product->get_attributes(), 'wc_attributes_array_filter_visible');

				$ret = '';
				if( !empty($attributes) ){
					$ret .= '<div class="gdlr-core-product-attributes" >';
					foreach ( $attributes as $slug => $attribute ) { $count--;
						$ret .= '<div class="gdlr-core-product-att" >';
						$ret .= '<span class="gdlr-head" >' . $attribute->get_name() . '</span>';
						$ret .= '<span class="gdlr-tail" >' . implode(',', $attribute->get_options()) . '</span>';
						$ret .= '</div>';

						if( $count == 0 ) break;
					}
					$ret .= '</div>';
				}
			   

				return  $ret;
			}

			// get product excerpt
			function product_excerpt( $excerpt_length ) {
				
				$post = get_post();

				if( empty($post) || post_password_required() ){ return ''; }
			
				$excerpt = $post->post_excerpt;
				if( empty($excerpt) ){
					$excerpt = get_the_content('');
					$excerpt = strip_shortcodes($excerpt);
					
					$excerpt = apply_filters('the_content', $excerpt);
					$excerpt = str_replace(']]>', ']]&gt;', $excerpt);
				}
				$excerpt_more = apply_filters('excerpt_more', '...');
				$excerpt = wp_trim_words($excerpt, $excerpt_length, $excerpt_more);
				
				$excerpt = apply_filters('wp_trim_excerpt', $excerpt, $post->post_excerpt);		
				$excerpt = apply_filters('get_the_excerpt', $excerpt);
				
				return '<div class="gdlr-core-product-excerpt" >' . $excerpt . '</div>';
			}	

			// product column
			function product_grid( $args ){

				$ret  = '<div class="gdlr-core-product-grid" >';
				$ret .= $this->product_thumbnail($args);
				
				$ret .= '<div class="gdlr-core-product-grid-content-wrap">';
				$ret .= $this->product_onsale();

				$ret .= '<div class="gdlr-core-product-grid-content">';
				$ret .= $this->product_title($args);

				$ret .= $this->product_price();
				$ret .= '</div>'; // gdlr-core-product-grid-content
				$ret .= '</div>'; // gdlr-core-product-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
			} 	

			function product_grid_style_2( $args ){

				$ret  = '<div class="gdlr-core-product-grid-2" >';
				$ret .= $this->product_thumbnail($args);
				
				$ret .= '<div class="gdlr-core-product-grid-content-wrap">';
				$ret .= '<div class="gdlr-core-product-grid-content">';
				$ret .= $this->product_title($args);

				if( empty($args['price-location']) || $args['price-location'] == 'after-title' ){
					$ret .= $this->product_price();
				}

				if( empty($args['enable-rating']) || $args['enable-rating'] == 'enable' ){
					$ret .= $this->product_rating();
				}
				$ret .= '</div>'; // gdlr-core-product-grid-content
				$ret .= '</div>'; // gdlr-core-product-grid-content-wrap
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
			} 	

			function product_grid_style_3( $args ){
				$extra_class  = ' gdlr-core-button-style-' . (empty($args['button-style'])? 'plain': $args['button-style']);
				$extra_class .= (!empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable')? ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element': '';
				if( $args['product-style'] == 'grid-3-with-border' ){
					$extra_class .= ' gdlr-core-with-border';
				}else if( $args['product-style'] == 'grid-3-without-frame' ){
					$extra_class .= ' gdlr-core-without-frame';
				}
				$args['product-style'] = 'grid-3';


				$frame_atts = array();
				if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
					$frame_atts['background-shadow-size'] = $args['frame-shadow-size'];
					$frame_atts['background-shadow-color'] = $args['frame-shadow-color'];
					$frame_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

					if( strpos($extra_class, 'gdlr-core-outer-frame-element') === false ){
						$extra_class .= ' gdlr-core-outer-frame-element';
					}
					
				}

				$border_size = empty($args['frame-border-size'])? '': $args['frame-border-size'];

				$ret  = '<div class="gdlr-core-product-grid-3 gdlr-core-item-mgb ' . esc_attr($extra_class) . '" ' . gdlr_core_esc_style($frame_atts) . ' >';
				
				if( !empty($args['button-style']) && $args['button-style'] == 'thumbnail' ){
					$args['product-style'] = 'grid-2';
				}
				$ret .= $this->product_thumbnail($args);
				
				$ret .= '<div class="gdlr-core-product-grid-content gdlr-core-skin-e-background clearfix" ' . gdlr_core_esc_style(array(
					'border-width' => empty($border_size)? '': "0px {$border_size} {$border_size}",
					'border-color' => empty($args['frame-border-color'])? '': $args['frame-border-color'],
					'padding' => empty($args['frame-padding'])? '': $args['frame-padding'],
				)) . '>';
				$ret .= '<div class="gdlr-core-product-grid-info clearfix" >';
				$ret .= $this->product_price();
				$ret .= $this->product_rating();
				$ret .= '</div>';
				
				$ret .= '<div class="gdlr-core-product-title-wrap" >';
				$ret .= $this->product_title($args);

				if( method_exists( 'YITH_WCWL_Shortcode', 'add_to_wishlist' ) && (empty($args['button-style']) || $args['button-style'] != 'thumbnail-modern') ){
					$ret .= YITH_WCWL_Shortcode::add_to_wishlist(
						array('product_id' => get_the_ID())
					);
				}
				$ret .= '</div>';

				$ret .= $this->product_attributes($args);

				if( empty($args['button-style']) || !in_array($args['button-style'], array('thumbnail', 'thumbnail-modern')) ){
					$ret .= $this->product_add_to_cart();
				}
				$ret .= '</div>'; // gdlr-core-product-grid-content
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
			} 

			function product_grid_style_4( $args ){

				$ret  = '<div class="gdlr-core-product-grid-4" >';
				$ret .= $this->product_thumbnail($args);
				
				$ret .= '<div class="gdlr-core-product-grid-content">';
				$ret .= $this->product_title($args);

				$ret .= '</div>'; // gdlr-core-product-grid-content
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
			} 	

			function product_grid_style_5( $args ){
				$extra_class = (!empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable')? ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element': '';

				$frame_atts = array();
				if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
					$frame_atts['background-shadow-size'] = $args['frame-shadow-size'];
					$frame_atts['background-shadow-color'] = $args['frame-shadow-color'];
					$frame_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

					if( strpos($extra_class, 'gdlr-core-outer-frame-element') === false ){
						$extra_class .= ' gdlr-core-outer-frame-element';
					}
					
				}
				if( !empty($args['border-radius']) ){
					$frame_atts['border-radius'] = $args['border-radius'];
				}
				$border_size = empty($args['frame-border-size'])? '': $args['frame-border-size'];

				$ret  = '<div class="gdlr-core-product-grid-5 gdlr-core-item-mgb ' . esc_attr($extra_class) . '" ' . gdlr_core_esc_style($frame_atts) . ' >';
				$ret .= $this->product_thumbnail($args);
				
				$ret .= '<div class="gdlr-core-product-grid-content gdlr-core-skin-e-background clearfix" ' . gdlr_core_esc_style(array(
					'border-width' => empty($border_size)? '': "0px {$border_size} {$border_size}",
					'border-color' => empty($args['frame-border-color'])? '': $args['frame-border-color'],
					'padding' => empty($args['frame-padding'])? '': $args['frame-padding'],
				)) . '>';
				$ret .= $this->product_onsale();
				$ret .= $this->product_title($args);

				if( !empty($args['excerpt-number']) ){
					$ret .= $this->product_excerpt($args['excerpt-number']);
				}
				
				$ret .= $this->product_attributes($args);
				$ret .= $this->product_add_to_cart();

				$ret .= '<div class="gdlr-core-product-grid-info clearfix" >';
				$ret .= $this->product_rating_with_count();
				$ret .= $this->product_price();
				$ret .= '</div>';
				$ret .= '</div>'; // gdlr-core-product-grid-content
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
			} 

			function product_grid_style_6( $args ){
				$extra_class = (!empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable')? ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element': '';

				$frame_atts = array();
				if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
					$frame_atts['background-shadow-size'] = $args['frame-shadow-size'];
					$frame_atts['background-shadow-color'] = $args['frame-shadow-color'];
					$frame_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

					if( strpos($extra_class, 'gdlr-core-outer-frame-element') === false ){
						$extra_class .= ' gdlr-core-outer-frame-element';
					}
					
				}
				if( !empty($args['border-radius']) ){
					$frame_atts['border-radius'] = $args['border-radius'];
				}
				$border_size = empty($args['frame-border-size'])? '': $args['frame-border-size'];

				$ret  = '<div class="gdlr-core-product-grid-6 gdlr-core-item-mgb ' . esc_attr($extra_class) . '" ' . gdlr_core_esc_style($frame_atts) . ' >';
				
				$ret .= $this->product_thumbnail($args);
				
				$ret .= '<div class="gdlr-core-product-grid-content gdlr-core-skin-e-background clearfix" ' . gdlr_core_esc_style(array(
					'border-width' => empty($border_size)? '': "0px {$border_size} {$border_size}",
					'border-color' => empty($args['frame-border-color'])? '': $args['frame-border-color'],
					'padding' => empty($args['frame-padding'])? '': $args['frame-padding'],
				)) . '>';
				
				$ret .= $this->product_title($args);
				$ret .= $this->product_price();
				$ret .= $this->product_rating_with_count();
				$ret .= $this->product_attributes($args);
				$ret .= $this->product_add_to_cart();
				
				$ret .= '</div>'; // gdlr-core-product-grid-content
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
			} 

			function product_box( $args ){
				global $product;

				$extra_class = (!empty($args['enable-move-up-shadow-effect']) && $args['enable-move-up-shadow-effect'] == 'enable')? ' gdlr-core-move-up-with-shadow gdlr-core-outer-frame-element': '';

				$frame_atts = array(
					'border-width' => empty($border_size)? '': "0px {$border_size} {$border_size}",
					'border-color' => empty($args['frame-border-color'])? '': $args['frame-border-color'],
					'padding' => empty($args['frame-padding'])? '': $args['frame-padding'],
					'background' => empty($args['box-background-color'])? '': $args['box-background-color']
				);

				if( !empty($args['frame-shadow-size']['size']) && !empty($args['frame-shadow-color']) && !empty($args['frame-shadow-opacity']) ){
					$frame_atts['background-shadow-size'] = $args['frame-shadow-size'];
					$frame_atts['background-shadow-color'] = $args['frame-shadow-color'];
					$frame_atts['background-shadow-opacity'] = $args['frame-shadow-opacity'];

					if( strpos($extra_class, 'gdlr-core-outer-frame-element') === false ){
						$extra_class .= ' gdlr-core-outer-frame-element';
					}
					
				}
				if( !empty($args['border-radius']) ){
					$frame_atts['border-radius'] = $args['border-radius'];
				}
				$border_size = empty($args['frame-border-size'])? '': $args['frame-border-size'];

				$ret  = '<div class="gdlr-core-product-box gdlr-core-item-mgb gdlr-core-js ' . esc_attr($extra_class) . '" ' . gdlr_core_esc_style($frame_atts) . ' data-sync-height="blog-item-' . esc_attr($this->product_item_id) . '" >';
				$ret .= $this->product_title($args);
				if( !empty($args['attribute-field']) ){
					$count = 0;
					$caption_fields = array_map('trim', explode('|', $args['attribute-field']));

					$ret .= '<div class="gdlr-core-product-attribute" ' . gdlr_core_esc_style(array(
						'color' => empty($args['product-caption-color'])? '': $args['product-caption-color']
					)) . ' >';
					foreach( $caption_fields as $field ){ 
						$attribute_text = $product->get_attribute($field);

						if( !empty($attribute_text) ){
							$count++;
							$ret .= (($count > 1)? ', ': '') . $attribute_text;
						}
					} 
					$ret .= '</div>';
				}
				if( !empty($args['variation-field']) ){
					$variations = $this->get_product_attribute_variation();
					
					$count = 0;
					$variation_fields =  array_map('trim', explode('|', $args['variation-field']));
					$ret .= '<div class="gdlr-core-product-attribute" ' . gdlr_core_esc_style(array(
						'color' => empty($args['product-caption-color'])? '': $args['product-caption-color']
					)) . ' >';
					foreach( $variation_fields as $field_slug ){
						
						if( !empty($variations[$field_slug]) ){
							$count++;
							$ret .= ($count > 1)? '<span class="gdlr-core-sep" >,</span> ': '';
							$ret .= '<span class="gdlr-core-head" >' . $field_slug . '</span>';
							$ret .= '<span class="gdlr-core-tail" >' . $variations[$field_slug] . '</span>';
						}
					}
					$ret .= '</div>';
				}

				$ret .= '<a class="gdlr-core-product-link" href="' . esc_attr(get_permalink()) . '" ></a>';
				$ret .= '</div>'; // gdlr-core-product-grid
				
				return $ret;
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
			
		} // gdlr_core_product_item
	} // class_exists